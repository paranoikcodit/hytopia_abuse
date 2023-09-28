use anyhow::anyhow;
use dialoguer::{theme::ColorfulTheme, Input};
use reqwest::Proxy;
use rnglib::{Language, RNG};
use rspasser::RsPasser;

const ANCHOR_URL: &str = "https://www.google.com/recaptcha/api2/anchor?ar=1&k=6Leqr00oAAAAAN3ItHtrGkMpHiOtENMkG87lq2fq&co=aHR0cHM6Ly9wcmVyZWdpc3Rlci5oeXRvcGlhLmNvbTo0NDM.&hl=ru&type=image&v=Ai7lOI0zKMDPHxlv62g7oMoJ&theme=dark&size=invisible&badge=bottomright&cb=bmwunnu5dq8d";

fn get_url(referral: Option<String>) -> String {
    if let Some(referral) = referral {
        format!("https://preregister.hytopia.com/{referral}/?_data=player-by-referrer")
    } else {
        String::from("https://preregister.hytopia.com/?_data=routes/_index")
    }
}

#[derive(Clone, Debug)]
struct Hytopia {
    client: reqwest::Client,
    username: String,
    email: String,
    referral: Option<String>,
}

impl Hytopia {
    pub fn new<T: Into<String>>(
        username: T,
        email: T,
        referral: Option<T>,
        proxy: Option<T>,
    ) -> Self {
        let mut builder = reqwest::Client::builder();

        if let Some(proxy) = proxy {
            let proxy = proxy.into();
            println!("{}", proxy.clone());
            builder = builder.proxy(Proxy::all(proxy).unwrap());
        }

        Self {
            username: username.into(),
            email: email.into(),
            referral: referral.map(|e| e.into()),
            client: builder.build().unwrap(),
        }
    }

    pub async fn register(&self) -> anyhow::Result<()> {
        let url = get_url(self.referral.clone());

        let token = self.solve_captcha(ANCHOR_URL.to_string()).await?;

        let response = self
            .client
            .post(url)
            .header(
                "content-type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .body(format!(
                "username={}&email={}&g-recaptcha-response={token}",
                self.username, self.email
            ))
            .send()
            .await?
            .text()
            .await?;

        println!("{response}");

        Ok(())
    }

    pub async fn solve_captcha(&self, anchor_url: String) -> anyhow::Result<String> {
        RsPasser::new().solve_captcha(anchor_url).await
    }

    pub async fn check_availability(&self) -> anyhow::Result<bool> {
        let url = get_url(self.referral.clone());

        let response = self
            .client
            .get(format!("{url}&username={}", self.username))
            .send()
            .await?
            .text()
            .await?;

        Ok(response.contains("usernameAvailable\":true"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let referral: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Your referral(or empty)")
        .default("".to_string())
        .interact()?;

    let referral = if referral == "" { None } else { Some(referral) };

    // let client = reqwest::Client::builder().cookie_store(true).build()?;

    let proxies = if matches!(tokio::fs::try_exists("proxies.txt").await, Ok(true)) {
        tokio::fs::read_to_string("proxies.txt")
            .await?
            .lines()
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    let emails = tokio::fs::read_to_string("emails.txt")
        .await
        .map_err(|e| anyhow!("emails.txt is not founded!"))?
        .lines()
        .map(String::from)
        .collect::<Vec<String>>();

    let clients = if proxies.is_empty() {
        emails
            .iter()
            .map(|email| {
                Hytopia::new(
                    RNG::new(&Language::Elven).unwrap().generate_name(),
                    email.clone(),
                    referral.clone(),
                    None,
                )
            })
            .collect::<Vec<Hytopia>>()
    } else {
        emails
            .iter()
            .zip(proxies)
            .map(|(email, proxy)| {
                Hytopia::new(
                    RNG::new(&Language::Elven).unwrap().generate_name(),
                    email.clone(),
                    referral.clone(),
                    Some(proxy),
                )
            })
            .collect::<Vec<Hytopia>>()
    };

    let handles = clients
        .iter()
        .cloned()
        .map(|client| {
            tokio::spawn(async move {
                client.register().await.unwrap();

                println!("Client {} registered", client.email);
            })
        })
        .collect::<Vec<_>>();

    futures::future::join_all(handles).await;

    // let client = Hytopia::new(
    //     "dosakfJDifjaisfjasiofjaSIO",
    //     "daskfiassjfasifjasi@gmail.com",
    //     Some("daskfiasjfasifjasi"),
    //     None,
    // );
    //
    // client.check_availability().await?;
    // client.register().await?;

    Ok(())

    // client.Ok(())
}
