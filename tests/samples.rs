#[cfg(test)]
mod tests {
    use nicebot::{NiceBot, Permission};
    use tokio::fs;

    #[tokio::test]
    async fn check_reddit() {
        let str = fs::read_to_string("./test-data/reddit.txt").await.unwrap();

        let robot = NiceBot::from(str);

        assert_eq!(robot.check("/"), Permission::Denied);
        assert_eq!(robot.check("/robots.txt"), Permission::Denied);
    }

    #[tokio::test]
    async fn check_tor() {
        let str = fs::read_to_string("./test-data/tor.txt").await.unwrap();

        let robot = NiceBot::from(str);

        assert_eq!(robot.check("/"), Permission::Unspecified);
        assert_eq!(robot.check("/java"), Permission::Unspecified);
        assert_eq!(robot.check("/javascript"), Permission::Denied);
        assert_eq!(robot.check("/javascriptandmore"), Permission::Denied);
        assert_eq!(robot.check("/javascript/andmore"), Permission::Denied);
    }

    #[tokio::test]
    async fn check_lib() {
        let str = fs::read_to_string("./test-data/lib.txt").await.unwrap();

        let robot = NiceBot::from(str);

        assert_eq!(robot.check("/"), Permission::Unspecified);
        assert_eq!(robot.check("/keywords"), Permission::Unspecified);
        assert_eq!(robot.check("/keywords/"), Permission::Denied);
        assert_eq!(robot.check("/search"), Permission::Denied);
        assert_eq!(robot.check("/search/"), Permission::Denied);
        assert_eq!(robot.check("/11111/source?at11111"), Permission::Denied);
        assert_eq!(robot.check("/source?at"), Permission::Denied);
    }

    #[tokio::test]
    async fn check_whatsapp() {
        let str = fs::read_to_string("./test-data/whatsapp.txt")
            .await
            .unwrap();

        let robot = NiceBot::from(str);

        assert_eq!(robot.check("/"), Permission::Allowed);
        assert_eq!(robot.check("/a"), Permission::Denied);
        assert_eq!(robot.check("/catalog"), Permission::Allowed);
        assert_eq!(robot.check("/catalog/"), Permission::Allowed);
        assert_eq!(robot.check("/catalogs"), Permission::Allowed);
        assert_eq!(robot.check("/🌐/"), Permission::Allowed);
    }
}
