use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use chat_core::{Chat, ChatType, Message};
use chat_server::AppState;
use futures::StreamExt;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::{net::TcpListener, time::sleep};

/*
test1:
    name: user 1 create chat
    steps:
        - signin
            email: yu@acme.org
            password: 123456
        - create_chat
            name: acme
            members: [1, 2]
        - create_message
            chat_id: 1
            content: hello
            files: [Cargo.toml]
*/

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

struct NotifyServer;

const WILD_ADDRESS: &str = "0.0.0.0:0"; // if you write like this, it will auto assign port

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let chat_server = ChatServer::new(state).await?;
    let db_url = tdb.url();
    NotifyServer::new(&db_url, &chat_server.token).await?;
    let chat = chat_server.create_chat().await?;
    let _message = chat_server.create_message(chat.id).await?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url.to_string();
        let app = notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDRESS).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move { axum::serve(listener, app.into_make_service()).await });

        let mut es = EventSource::get(format!("http://{}/events?token={}", addr, token));

        tokio::spawn(async move {
            // if it is a stream, we need to use StreamExt, you can use futures::StreamExt, and then use next() to get the next event
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(chat.name.as_ref().unwrap(), "acme");
                            assert_eq!(chat.members, vec![1, 2]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }
                        "NewMessage" => {
                            let message: Message = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(message.content, "hello");
                            assert_eq!(message.files.len(), 1);
                            assert_eq!(message.sender_id, 1);
                        }
                        _ => panic!("Unknown event: {}", message.event),
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        es.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}

impl ChatServer {
    async fn new(state: AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDRESS).await?;
        let addr = listener.local_addr()?;

        // because let it not block, so we need to use tokio::spawn, otherwise it will block the main thread
        tokio::spawn(async move { axum::serve(listener, app.into_make_service()).await });

        let client = reqwest::Client::new();

        let mut ret = Self {
            addr,
            token: "".to_string(),
            client,
        };

        ret.token = ret.signin().await?;

        Ok(ret)
    }

    async fn signin(&self) -> Result<String> {
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(r#"{"email":"yu@acme.org","password":"123456"}"#)
            .send()
            .await?;
        assert_eq!(res.status(), 200);
        let ret: AuthToken = res.json().await?;
        Ok(ret.token)
    }

    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(r#"{"name":"acme", "members": [1,2], "public": false}"#)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name.as_ref().unwrap(), "acme");
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(chat)
    }

    async fn create_message(&self, chat_id: i64) -> Result<Message> {
        // upload file
        // include_bytes! is a macro that reads the file at compile time , it does not IO operation
        // Part::file has a method that can read the file at runtime, it is a IO operation
        // IO operation is not allowed in const, so we need to use include_bytes! to read the file at compile time
        // note: if you test in github action, we should use include_bytes! to read the file at compile time
        // if not, you use Part::file to read the file at runtime, github action perhaps find no file or directory
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);

        let res = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let ret: Vec<String> = res.json().await?;

        let body = serde_json::to_string(&json!({
            "content": "hello",
            "files": ret
        }))?;

        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let message: Message = res.json().await?;
        assert_eq!(message.content, "hello");
        assert_eq!(message.files, ret);
        assert_eq!(message.sender_id, 1);
        assert_eq!(message.chat_id, chat_id);
        Ok(message)
    }
}
