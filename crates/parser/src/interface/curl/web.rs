/// This module defines the WebClient struct, which provides methods for fetching HTML content.
/// 
/// TODO : There is a prototype code that is meant to perform a connexion and add to cart operation on the Stade Rochelais billetterie,
/// but it is currently not functional and needs to be completed and tested.
use curl::easy::Easy;
use crate::controller::html_extract::FetchHtml;
use crate::core::seat::SeatAction;

/// WebClient is a struct that implements the FetchHtml trait, allowing it to fetch HTML content from a URL
pub struct WebClient {
    pub cookie_jar: Option<String>,
}

/// Webclient implementation functions for creating new instances with or without session cookies
impl WebClient {
    pub fn new() -> Self {
        Self { cookie_jar: None }
    }

    pub fn with_session(cookie_jar: &str) -> Self {
        Self { cookie_jar: Some(cookie_jar.to_string()) }
    }
}

/// Implements the FetchHtml trait for WebClient, allowing it to fetch HTML content,
/// add a seat to the cart, -> not implemented yet
/// and connect to the shop with credentials to add a seat to the cart. -> not implemented yet
impl FetchHtml for WebClient {
    fn get_html(&self, url: &str) -> Result<String, String> {
        fetch_html(url, self.cookie_jar.as_deref()).map_err(|e| e.to_string())
    }

    fn add_to_cart(&self, action: &SeatAction) -> Result<(), String> {
        add_to_cart(action, self.cookie_jar.as_deref()).map_err(|e| e.to_string())
    }

    fn connect_and_add_to_cart(&self, email: &str, password: &str, action: &SeatAction) -> Result<(), String> {
        connect_and_add_to_cart(email, password, action).map_err(|e| e.to_string())
    }
}

/// Fetches HTML content from the given URL, optionally using a cookie jar for session management.
/// 
/// # Arguments
/// * `url` - The URL from which to fetch HTML content
/// * `cookie_jar` - An optional path to a cookie jar file for managing session cookies
/// # Returns
/// A Result containing the fetched HTML content as a String, or an error if the fetch operation fails
fn fetch_html(url: &str, cookie_jar: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut easy = Easy::new();
    easy.url(url)?;
    if let Some(jar) = cookie_jar {
        easy.cookie_file(jar)?;
        easy.cookie_jar(jar)?;
    }
    easy.follow_location(true)?;
    let mut html = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            html.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    Ok(String::from_utf8(html)?)
}

pub fn connect_to_shop(
    _email: &str,
    _password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
    // let cookie_jar = "/tmp/sr_session.jar";
    // let _ = std::fs::remove_file(cookie_jar); // repart d'une session vierge

    // println!("=== Étape 1 : GET page de login (récupère form_build_id + cookie anonyme) ===");
    // let login_page = fetch_with_jar("https://billetterie.staderochelais.com/fr/user/login", cookie_jar)
    //     .expect("Impossible de charger la page de login");
    // let form_build_id = extract_form_build_id(&login_page)
    //     .expect("form_build_id introuvable dans la page de login");
    // println!("form_build_id étape 1 : {}", form_build_id);

    // println!("\n=== Étape 2 : POST email (Continuer) → récupère formulaire mot de passe ===");
    // let step2_html = post_email_step(&email, &form_build_id, cookie_jar)
    //     .expect("Erreur lors du POST email");
    // if step2_html.contains("/register") {
    //     eprintln!("❌ Compte introuvable pour cet email — vérifier SHOP_EMAIL");
    //     return Err("Compte introuvable pour cet email".into());
    // }
    // let form_build_id2 = extract_form_build_id(&step2_html)
    //     .expect("form_build_id introuvable dans la réponse email");
    // println!("form_build_id étape 2 : {}", form_build_id2);

    // println!("\n=== Étape 3 : POST mot de passe ===");
    // let login_url = "https://billetterie.staderochelais.com/fr/user/login";
    // let response = post_password_step(&email, &password, &form_build_id2, login_url, cookie_jar)
    //     .expect("Erreur lors du POST mot de passe");
    // println!("Réponse (200 premiers chars) : {}", &response[..response.len().min(200)]);

    // println!("\n=== Étape 4 : GET panier (vérifie que la session est authentifiée) ===");
    // let basket = fetch_with_jar("https://billetterie.staderochelais.com/fr/basket", cookie_jar)
    //     .expect("Impossible de charger le panier");
    // if basket.contains("user-logged-in") {
    //     println!("✅ Connecté — session authentifiée valide");
    // } else {
    //     println!("❌ Non connecté — vérifier email/password ou présence d'un captcha");
    //     return Err("Non connecté — vérifier email/password ou présence d'un captcha".into());
    // }
    // println!("Cookie jar écrit dans : {}", cookie_jar);
    // Ok(cookie_jar.to_string())
}

#[allow(dead_code)]
fn fetch_with_jar(_url: &str, _cookie_jar: &str) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
    // use curl::easy::Easy;
    // let mut easy = Easy::new();
    // easy.url(url)?;
    // easy.cookie_file(cookie_jar)?;
    // easy.cookie_jar(cookie_jar)?;
    // easy.follow_location(true)?;
    // let mut html = Vec::new();
    // {
    //     let mut transfer = easy.transfer();
    //     transfer.write_function(|data| { html.extend_from_slice(data); Ok(data.len()) })?;
    //     transfer.perform()?;
    // }
    // Ok(String::from_utf8(html)?)
}

pub fn extract_form_token(_html: &str) -> Option<String> {
    todo!()
    // Forme HTML brut : name="form_token" value="XXXX"
    // if let Some(pos) = html.find("name=\"form_token\"") {
    //     let after = &html[pos..];
    //     if let Some(val_pos) = after.find("value=\"") {
    //         let val_start = val_pos + 7;
    //         if let Some(val_end) = after[val_start..].find('"') {
    //             return Some(after[val_start..val_start + val_end].to_string());
    //         }
    //     }
    // }
    // // Forme JSON-échappée (BigPipe) : form_token\u0022 value=\u0022XXXX\u0022
    // if let Some(pos) = html.find("form_token\\u0022 value=\\u0022") {
    //     let start = pos + "form_token\\u0022 value=\\u0022".len();
    //     if let Some(end) = html[start..].find("\\u0022") {
    //         return Some(html[start..start + end].to_string());
    //     }
    // }

}

pub fn extract_form_build_id(_html: &str) -> Option<String> {
    todo!()
    // // Forme HTML brut : name="form_build_id" value="form-XXXX"
    // if let Some(pos) = html.find("name=\"form_build_id\"") {
    //     let after = &html[pos..];
    //     if let Some(val_pos) = after.find("value=\"") {
    //         let val_start = val_pos + 7;
    //         if let Some(val_end) = after[val_start..].find('"') {
    //             return Some(after[val_start..val_start + val_end].to_string());
    //         }
    //     }
    // }
    // // Forme JSON-échappée (BigPipe/AJAX) : name\u0022 value=\u0022form-XXXX\u0022
    // if let Some(pos) = html.find("form_build_id\\u0022 value=\\u0022") {
    //     let start = pos + "form_build_id\\u0022 value=\\u0022".len();
    //     if let Some(end) = html[start..].find("\\u0022") {
    //         return Some(html[start..start + end].to_string());
    //     }
    // }
    // None
}

#[allow(dead_code)]
fn post_email_step(
    _email: &str,
    _form_build_id: &str,
    _cookie_jar: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
    // use curl::easy::{Easy, List};
    // let login_url = "https://billetterie.staderochelais.com/fr/user/login";
    // let body = format!(
    //     "name={}&form_build_id={}&form_id=user_login_form&op=Continuer",
    //     urlencode(email),
    //     form_build_id,
    // );
    // let mut easy = Easy::new();
    // easy.url(login_url)?;
    // easy.post(true)?;
    // easy.post_fields_copy(body.as_bytes())?;
    // easy.cookie_file(cookie_jar)?;
    // easy.cookie_jar(cookie_jar)?;
    // easy.follow_location(true)?;
    // let mut headers = List::new();
    // headers.append("Content-Type: application/x-www-form-urlencoded")?;
    // let referer = format!("Referer: {}", login_url);
    // headers.append(&referer)?;
    // easy.http_headers(headers)?;
    // let mut response = Vec::new();
    // {
    //     let mut transfer = easy.transfer();
    //     transfer.write_function(|data| { response.extend_from_slice(data); Ok(data.len()) })?;
    //     transfer.perform()?;
    // }
    // Ok(String::from_utf8(response)?)
}

#[allow(dead_code)]
fn post_password_step(
    _email: &str,
    _password: &str,
    _form_build_id: &str,
    _url: &str,
    _cookie_jar: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
//     use curl::easy::{Easy, List};
//     let body = format!(
//         "name={}&pass={}&form_build_id={}&form_id=user_login_form&op=Se+connecter",
//         urlencode(email),
//         urlencode(password),
//         form_build_id,
//     );
//     let mut easy = Easy::new();
//     easy.url(url)?;
//     easy.post(true)?;
//     easy.post_fields_copy(body.as_bytes())?;
//     easy.cookie_file(cookie_jar)?;
//     easy.cookie_jar(cookie_jar)?;
//     easy.follow_location(true)?;
//     let mut headers = List::new();
//     headers.append("Content-Type: application/x-www-form-urlencoded")?;
//     let referer = format!("Referer: {}", url);
//     headers.append(&referer)?;
//     easy.http_headers(headers)?;
//     let mut response = Vec::new();
//     {
//         let mut transfer = easy.transfer();
//         transfer.write_function(|data| { response.extend_from_slice(data); Ok(data.len()) })?;
//         transfer.perform()?;
//     }
//     Ok(String::from_utf8(response)?)
// }

// fn urlencode(s: &str) -> String {
//     s.chars().flat_map(|c| match c {
//         'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => vec![c],
//         '@' => vec!['%', '4', '0'],
//         c => {
//             let encoded = format!("%{:02X}", c as u32);
//             encoded.chars().collect()
//         }
//     }).collect()
}

pub fn connect_and_add_to_cart(
    _email: &str,
    _password: &str,
    _action: &SeatAction,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
    // let cookie_jar = connect_to_shop(email, password)?;

    // let base_url = "https://billetterie.staderochelais.com";
    // let resale_page_url = format!("{}{}", base_url, action.ajax_url.split('?').next().unwrap_or(""));
    // let auth_client = WebClient::with_session(&cookie_jar);
    // let fresh_html = auth_client.get_html(&resale_page_url)
    //     .map_err(|e| e.to_string())?;

    // let fresh_form_build_id = extract_form_build_id(&fresh_html)
    //     .ok_or("form_build_id introuvable dans la page de revente")?;
    // let fresh_form_token = extract_form_token(&fresh_html)
    //     .ok_or("form_token introuvable dans la page de revente")?;

    // let fresh_action = SeatAction {
    //     form_build_id: fresh_form_build_id,
    //     form_token: fresh_form_token,
    //     ..action.clone()
    // };

    // add_to_cart(&fresh_action, Some(&cookie_jar))
}


pub fn add_to_cart(
    _action: &SeatAction,
    _cookie_jar: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
//     let base_url = "https://billetterie.staderochelais.com";
//     let url = format!("{}{}&_wrapper_format=drupal_ajax", base_url, action.ajax_url);

//     let body = format!(
//         "nb_tickets_selector=0\
//         &price_slider={min}%2C{max}\
//         &sort=price-asc\
//         &selected_tickets_{pack_id}={ticket_id}\
//         &tickets_selected=\
//         &resale_pack_selected={pack_id}\
//         &form_build_id={form_build_id}\
//         &form_token={form_token}\
//         &form_id=hubber_resale_add_to_cart_form\
//         &_triggering_element_name=add_pack\
//         &_triggering_element_value=Ajouter+%C3%A0+mon+panier\
//         &_drupal_ajax=1\
//         &ajax_page_state%5Btheme%5D=hubber_reference8\
//         &ajax_page_state%5Btheme_token%5D=\
//         &ajax_page_state%5Blibraries%5D={libraries}",
//         min = action.price_min,
//         max = action.price_max,
//         pack_id = action.pack_id,
//         ticket_id = action.ticket_id,
//         form_build_id = action.form_build_id,
//         form_token = action.form_token,
//         libraries = action.libraries,
//     );

//     let mut easy = Easy::new();
//     easy.url(&url)?;
//     easy.post(true)?;
//     easy.post_fields_copy(body.as_bytes())?;
//     if let Some(jar) = cookie_jar {
//         easy.cookie_file(jar)?;
//         easy.cookie_jar(jar)?;
//     }

//     let mut headers = List::new();
//     headers.append("Accept: application/json, text/javascript, */*; q=0.01")?;
//     headers.append("Content-Type: application/x-www-form-urlencoded; charset=UTF-8")?;
//     headers.append("X-Requested-With: XMLHttpRequest")?;

//     headers.append(&format!("Referer: {}{}", base_url, action.ajax_url.split('?').next().unwrap_or("")))?;
//     easy.http_headers(headers)?;

//     let mut response = Vec::new();
//     {
//         let mut transfer = easy.transfer();
//         transfer.write_function(|data| {
//             response.extend_from_slice(data);
//             Ok(data.len())
//         })?;
//         transfer.perform()?;
//     }

//     let json_str = String::from_utf8(response)?;
//     if json_str.contains("p\u{e9}rim\u{e9}") || json_str.contains("p%C3%A9rim%C3%A9") || (json_str.contains("alert-danger") && json_str.contains("formulaire")) {
//         return Err("Formulaire p\u{e9}rim\u{e9} \u{2014} form_build_id expir\u{e9} ou session incorrecte".into());
//     }
//     println!("{:#?}", json_str);
//     Ok(())
}

