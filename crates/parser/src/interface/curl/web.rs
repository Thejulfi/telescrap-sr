use curl::easy::{Easy, List};
use crate::controller::html_extract::FetchHtml;
use crate::core::seat::SeatAction;

pub struct WebClient;

impl FetchHtml for WebClient {
    fn get_html(&self, url: &str) -> Result<String, String> {
        fetch_html(url).map_err(|e| e.to_string())
    }

    fn add_to_cart(&self, action: &SeatAction) -> Result<(), String> {
        add_to_cart(action).map_err(|e| e.to_string())
    }
}

fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut easy = Easy::new();
    easy.url(url)?;
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

pub fn add_to_cart(
    action: &SeatAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "https://billetterie.staderochelais.com";
    let url = format!("{}{}&_wrapper_format=drupal_ajax", base_url, action.ajax_url);

    let body = format!(
        "nb_tickets_selector=0\
        &price_slider={min}%2C{max}\
        &sort=price-asc\
        &selected_tickets_{pack_id}={ticket_id}\
        &tickets_selected=\
        &resale_pack_selected={pack_id}\
        &form_build_id={form_build_id}\
        &form_id=hubber_resale_add_to_cart_form\
        &_triggering_element_name=add_pack\
        &_triggering_element_value=Ajouter+%C3%A0+mon+panier\
        &_drupal_ajax=1\
        &ajax_page_state%5Btheme%5D=hubber_reference8\
        &ajax_page_state%5Btheme_token%5D=\
        &ajax_page_state%5Blibraries%5D={libraries}",
        min = action.price_min,
        max = action.price_max,
        pack_id = action.pack_id,
        ticket_id = action.ticket_id,
        form_build_id = action.form_build_id,
        libraries = action.libraries,
    );

    let mut easy = Easy::new();
    easy.url(&url)?;
    easy.post(true)?;
    easy.post_fields_copy(body.as_bytes())?;

    let mut headers = List::new();
    headers.append("Accept: application/json, text/javascript, */*; q=0.01")?;
    headers.append("Content-Type: application/x-www-form-urlencoded; charset=UTF-8")?;
    headers.append("X-Requested-With: XMLHttpRequest")?;

    headers.append(&format!("Referer: {}{}", base_url, action.ajax_url.split('?').next().unwrap_or("")))?;
    easy.http_headers(headers)?;

    let mut response = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let json_str = String::from_utf8(response)?;
    println!("{:#?}", json_str);

    Ok(())
}

