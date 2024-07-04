pub const MOCK_USER: &str = r#"
{
  "cmd": "DISPATCH",
  "data": {
    "v": 1,
    "config": {
      "cdn_host": "cdn.discordapp.com",
      "api_endpoint": "//discord.com/api",
      "environment": "production"
    },
    "user": {
      "id": "1045800378228281345",
      "username": "arrpc",
      "discriminator": "0",
      "global_name": "arRPC",
      "avatar": "cfefa4d9839fb4bdf030f91c2a13e95c",
      "avatar_decoration_data": null,
      "bot": false,
      "flags": 0,
      "premium_type": 0
    }
  },
  "evt": "READY",
  "nonce": null
}
"#;
