# discordbot-rust
`local_lavalink_server branch` is dedicated for self-sufficient `Lavalink` server version.
<br><br>In other words, this version is basically the same as `online_lavalink_server branch`, and will be updated along with in future commits.
<br>
One exception being that `Lavalink` server in `local branch` is run locally, while `online branch` uses free online [`Lavalink` server](https://support.something.host/en/article/lavalink-hosting-okm26z/), credited to [SomethingHost](https://something.host).
<br><br>
It is preferable to host bot online (on Heroku, for instance) using `online_lavalink_server branch` version, as it would consume much less RAM than `local branch`.
<br>
If stable connection and smooth experience is prioritized, go for `local_lavalink_server branch` version.
<br><br>
Besides Rust, Java (see more on [Lavalink Github repository](https://github.com/freyacodes/Lavalink)) and NodeJS is also required.