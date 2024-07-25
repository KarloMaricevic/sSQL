use clap::{Arg, Command};

pub struct Config {
    pub host: String,
    pub port: i32,
}

impl Config {
    pub fn build() -> Result<Self, String> {
        let matches = Command::new("sSQL CLI")
            .version("1.0")
            .about("This program is sending commands to sSQL server over TCP connection")
            .disable_help_flag(true)
            .arg(
                Arg::new("host")
                    .help("The host address")
                    .short('h')
                    .long("host")
                    .value_name("HOST")
                    .default_value("127.0.0.1"),
            )
            .arg(
                Arg::new("port")
                    .help("The port number")
                    .short('p')
                    .long("port")
                    .value_name("PORT")
                    .default_value("3307"),
            )
            .get_matches();

        let host = matches
            .get_one::<String>("host")
            .ok_or("Host argument couldnt be parsed")?
            .to_string();
        let port = matches
            .get_one::<String>("port")
            .ok_or("Port argument couldnt be parsed")?
            .parse::<i32>()
            .unwrap();

        Ok(Config { host, port })
    }
}
