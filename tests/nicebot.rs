#[cfg(test)]
mod tests {
    use nicebot::{NiceBot, Permission};

    #[test]
    fn from_str() {
        const INPUT: &str = r#"
            User-agent: *
            Allow: /abc
        "#;

        let bot = NiceBot::from(INPUT);

        assert_eq!(bot.check("/xyz"), Permission::Unspecified);
        assert_eq!(bot.check("/abc"), Permission::Allowed);
    }

    #[test]
    fn from_file() {
        let input = std::fs::File::open("test-data/tor.txt").unwrap();

        let bot = NiceBot::from(input);

        assert_eq!(bot.check("/sass"), Permission::Unspecified);
        assert_eq!(bot.check("/scss"), Permission::Denied);
    }

    #[test]
    fn from_reader() {
        let input = std::fs::File::open("test-data/tor.txt").unwrap();
        let reader = std::io::BufReader::new(input);

        let bot = NiceBot::from(reader);

        assert_eq!(bot.check("/sass"), Permission::Unspecified);
        assert_eq!(bot.check("/scss"), Permission::Denied);
    }

    #[cfg(feature = "async-tokio")]
    #[test]
    fn from_file_tokio() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let input = tokio::fs::File::open("test-data/tor.txt").await.unwrap();

            let bot = NiceBot::from_file_tokio(input).await;

            assert_eq!(bot.check("/sass"), Permission::Unspecified);
            assert_eq!(bot.check("/scss"), Permission::Denied);
        });
    }

    #[cfg(feature = "async-async-std")]
    #[test]
    fn from_file_asyncstd() {
        async_std::task::block_on(async {
            let input = async_std::fs::File::open("test-data/tor.txt")
                .await
                .unwrap();

            let bot = NiceBot::from_file_asyncstd(input).await;

            assert_eq!(bot.check("/sass"), Permission::Unspecified);
            assert_eq!(bot.check("/scss"), Permission::Denied);
        });
    }

    #[cfg(feature = "async-smol")]
    #[test]
    fn from_file_smol() {
        smol::block_on(async {
            let input = smol::fs::File::open("test-data/tor.txt").await.unwrap();

            let bot = NiceBot::from_file_smol(input).await;

            assert_eq!(bot.check("/sass"), Permission::Unspecified);
            assert_eq!(bot.check("/scss"), Permission::Denied);
        });
    }
}
