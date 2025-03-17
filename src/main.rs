mod web_server;

use std::env;
use std::sync::LazyLock;
use base64::Engine;
use teloxide::dispatching::dialogue::{SqliteStorage,serializer:: Json,ErasedStorage,Storage};
use teloxide::dispatching::{dialogue, HandlerExt, MessageFilterExt};
use teloxide::net::Download;
use teloxide::types::{BotCommand,InlineKeyboardButton, InlineKeyboardMarkup,ParseMode};
use teloxide::prelude::*;
use teloxide::utils::{command::BotCommands,markdown};
use std::time::{SystemTime,UNIX_EPOCH};
pub type Error = Box<dyn std::error::Error + Send + Sync>;
type MyDialogue = Dialogue<State,ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;





#[derive(Debug,BotCommands,Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
enum MyCommands {
    Login,
    Exit
} 
static KEY: LazyLock<String> = LazyLock::new(|| {
    env::var("KEY").expect("KEY must be set")  
});
static DOMAIN: LazyLock<String>=LazyLock::new(|| {
    env::var("DOMAIN").expect("DOMAIN must be set")});
#[tokio::main]
async fn main()->Result<(),Error> {
    dotenv::dotenv().ok();

    static BOT_API: LazyLock<String>=LazyLock::new(|| {env::var("BOT_API").expect("BOT_API must be set")});


    let bot=Bot::new(&*BOT_API);
    {
        bot.set_my_commands(vec![BotCommand::new("login","登录"),BotCommand::new("exit","退出登录状态")]).await?;        
        // let conn=Connection::open("member.db")?;
        // conn.execute("create table if not exists member(id INTEGER PRIMARY KEY,user integer UNIQUE)", [])?;
    }
    let delete_button=vec![InlineKeyboardButton::callback("delete", "delete_file")];

    let storage:MyStorage=SqliteStorage::open("db.sqlite", Json).await?.erase();
    let message_handler=Update::filter_message()
        .branch(dptree::case![State::Start]
            .branch(dptree::entry().filter_command::<MyCommands>()
                    .branch(dptree::case![MyCommands::Login].endpoint(user_login))
                    )
            .branch(Message::filter_text().endpoint(handle_msg))
            .branch(Message::filter_photo().endpoint(handle_msg))
            .branch(Message::filter_document().endpoint(handle_msg))
        )
        .branch(dptree::case![State::Login].branch(Message::filter_text().endpoint(key_verify))
        )
        .branch(dptree::case![State::Logined]
            .branch(dptree::entry().filter_command::<MyCommands>()
                    .branch(dptree::case![MyCommands::Exit].endpoint(exit_login)))
            .branch(Message::filter_photo().endpoint(photo_download))
            .branch(Message::filter_document().endpoint(file_download))
        );
    let callback_query_handler=Update::filter_callback_query()
        .branch(dptree::case![State::Logined].endpoint(callback_query_handle));
    let schema=dialogue::enter::<Update,ErasedStorage<State>,State,_>()
        .branch(message_handler)
        .branch(callback_query_handler);
    let tg_bot=tokio::spawn(
        async move{
            Dispatcher::builder(bot, schema).dependencies(dptree::deps![storage,delete_button]).build().dispatch().await;
        }
    
    );

    tokio::try_join!(tg_bot,web_server::web_server().await).unwrap();

    Result::Ok(())
}

#[derive(Clone,Default,serde::Serialize,serde::Deserialize)]
enum State {
    #[default]
    Start,
    Login,
    Logined

}

async fn  callback_query_handle(bot:Bot,q:CallbackQuery)->Result<(),Error> {
    bot.answer_callback_query(q.id).await?;
    let msg=q.message.unwrap().clone();
    let text=msg.regular_message().unwrap().text().unwrap();
    // println!("{text}");
    bot.edit_message_text(q.from.id, msg.id(),
        markdown::strike(text.replace(":", "\\:").replace("/", "\\/").replace("=", "\\=").replace(".", "\\.").as_str())).parse_mode(ParseMode::MarkdownV2).await?;
    let path=String::from_utf8(web_server::URL_SAFE.decode(text.split("/").last().unwrap()).unwrap()).unwrap();
    // println!("{path}");
    tokio::fs::remove_file(path).await?;
    Result::Ok(())
}


async fn exit_login(bot:Bot,msg:Message,dialogue:MyDialogue)->Result<(),Error> {
    create_or_delete_dir(msg.chat.id,0).await?;
    bot.send_message(msg.chat.id, "已经退出登录状态").await?;
    dialogue.update(State::Start).await?;
    Result::Ok(())
}


async fn user_login(bot:Bot,msg:Message,dialogue:MyDialogue)->Result<(),Error> {
    bot.send_message(msg.chat.id, "请输入登录密钥").await?;
    dialogue.update(State::Login).await?;

    Result::Ok(())
}

async fn create_or_delete_dir(id:ChatId,action:i32)->Result<(),Error> {

    
    if action==1 {
        tokio::fs::create_dir_all(format!("img/{id}")).await?;    
    }
    else if action==0{
        tokio::fs::remove_dir_all(format!("img/{id}")).await?;
    }
    else {
        
    }
    Result::Ok(())

}

async fn key_verify(bot:Bot,msg:Message,dialogue:MyDialogue)->Result<(),Error>{
    match msg.text(){
        Some(key)=>{
            if &key.to_string()==&*KEY {
                create_or_delete_dir(msg.chat.id,1).await?;
                bot.send_message(msg.chat.id, "登录成功").await?;
                dialogue.update(State::Logined).await?
            }
            else {
                bot.send_message(msg.chat.id, "登录失败").await?;
            }
        },
        None=>{}
        
    }
    Result::Ok(())
}


async fn handle_msg(bot:Bot,msg:Message)->Result<(),Error>{
    bot.send_message(msg.chat.id, "你还没有登录,请使用/login登录").await?;
    Result::Ok(())
}



async fn photo_download(bot:Bot,msg:Message,button:Vec<InlineKeyboardButton>)->Result<(),Error> {
    // print!("收到图片");
    // bot.send_message(msg.chat.id, "text").await?;
    match msg.photo() {
        None=>{},
        Some(photo)=>{
            let file_id=photo.last().unwrap().file.id.clone();
            let file_path=bot.get_file(&file_id).await?.path;
            let img_name=format!("{}_{}",msg.chat.id,SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string());;
            let mut dst=tokio::fs::File::create(format!("img/{}/{}.bmp",msg.chat.id,img_name)).await?; 
             
            bot.download_file(&file_path,&mut dst ).await?;
            bot.send_message(msg.chat.id, format!("{}/{}",
                &*DOMAIN,
                web_server::URL_SAFE.encode(format!("img/{}/{}.bmp",msg.chat.id,img_name))
                )).reply_markup(InlineKeyboardMarkup::new([button])).await?;
        }
    }
    // bot.send_message(msg.chat.id, "下载成功").await?;
    Result::Ok(())
}

async fn file_download(bot:Bot,msg:Message,button:Vec<InlineKeyboardButton>)->Result<(),Error> {
    match msg.document() {
        None=>{},
        Some(doc)=>{
            let file_id=doc.file.id.clone();
            let suffix_name=doc.file_name.clone().unwrap().split(".").last().unwrap().to_string();
            let file_path=bot.get_file(&file_id).await?.path;
            let file_name=format!("{}_{}",msg.chat.id,SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string());
            let mut dst=tokio::fs::File::create(format!("img/{}/{}.{}",msg.chat.id,file_name,suffix_name)).await?; 
             
            bot.download_file(&file_path,&mut dst ).await?;
            bot.send_message(msg.chat.id, format!("{}/{}",
            &*DOMAIN,
            web_server::URL_SAFE.encode(format!("img/{}/{}.{}",
           msg.chat.id,file_name,suffix_name))
            )).reply_markup(InlineKeyboardMarkup::new([button])).await?;
        }
    }
    Result::Ok(())
}



// async fn verify_user(chat_id:ChatId)->Result<bool,Error> {
//     let conn=Connection::open("member.db")?;
//     let res=conn.query_row("select exists(select user from member where user=?1)", 
//         params![chat_id.0],|row|row.get::<_,bool>(0));
//     match res {
//         Ok(_)=>{
//             Result::Ok(true)
//         },
//         Err(_)=>{
//             Result::Ok(false)
//         }
//     }
// }
// async fn add_user(chat_id:ChatId)->Result<(),Error> {

//     let conn=Connection::open("member.db")?;
//     conn.execute("insert into member(user) values(?1)", params![chat_id.0])?;
//     Result::Ok(())
// }