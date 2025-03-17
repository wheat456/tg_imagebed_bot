# steps
1. 在linux创建一个文件夹，并在其中创建`.env`文件
```
BOT_API="你的bot token"
KEY="dsadsa" #自定义密钥，随便一个字符串
DOMAIN="127.0.0.1:8080" # 域名比如 www.google.com
```
2. 运行 `curl -LO https://github.com/wheat456/tg_imagebed_bot/releases/download/0.0.2/imagebed_bot_v0.0.2 && chmod +x imagebed_bot_v0.0.2 && nohup ./imagebed_bot_v0.0.2 &`

# attention
- web服务在8080端口，推荐使用cloudflare 源规则使用
- 图片是按照tg id 来分类的，都在img文件夹下

