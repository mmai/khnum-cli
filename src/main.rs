use clap::{Arg, App};
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let matches = App::new("Khnum")
        .version("0.1.0")
        .author("Henri Bourcereau <hbourcereau@gmail.com>")
        .about("Khnum web framework command line interface")
        .arg(Arg::with_name("command")
                 .required(true)
                 .takes_value(true)
                 .index(1)
                 .help("command to execute"))
        .get_matches();
    let cde = matches.value_of("command").unwrap();
    println!("{}", cde);
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => {
            match length {
                Some(len) => ProgressBar::new(len),
                None => ProgressBar::new_spinner(),
            }
        }
    };

    bar.set_message(msg);
    match length.is_some() {
        true => bar
            .set_style(ProgressStyle::default_bar()
                .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}")
                .progress_chars("=> ")),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };

    bar
}

fn download(target: &str, quiet_mode: bool) -> Result<(), Box<::std::error::Error>> {
    // parse url
    let url = parse_url(target)?;
    let client = Client::new().unwrap();
    let mut resp = client.get(url)?
        .send()
        .unwrap();
    print(format!("HTTP request sent... {}",
                  style(format!("{}", resp.status())).green()),
          quiet_mode);
    if resp.status().is_success() {

        let headers = resp.headers().clone();
        let ct_len = headers.get::<ContentLength>().map(|ct_len| **ct_len);

        let ct_type = headers.get::<ContentType>().unwrap();

        match ct_len {
            Some(len) => {
                print(format!("Length: {} ({})",
                      style(len).green(),
                      style(format!("{}", HumanBytes(len))).red()),
                    quiet_mode);
            },
            None => {
                print(format!("Length: {}", style("unknown").red()), quiet_mode); 
            },
        }

        print(format!("Type: {}", style(ct_type).green()), quiet_mode);

        let fname = target.split("/").last().unwrap();

        print(format!("Saving to: {}", style(fname).green()), quiet_mode);

        let chunk_size = match ct_len {
            Some(x) => x as usize / 99,
            None => 1024usize, // default chunk size
        };

        let mut buf = Vec::new();

        let bar = create_progress_bar(quiet_mode, fname, ct_len);

        loop {
            let mut buffer = vec![0; chunk_size];
            let bcount = resp.read(&mut buffer[..]).unwrap();
            buffer.truncate(bcount);
            if !buffer.is_empty() {
                buf.extend(buffer.into_boxed_slice()
                               .into_vec()
                               .iter()
                               .cloned());
                bar.inc(bcount as u64);
            } else {
                break;
            }
        }

        bar.finish();

        save_to_file(&mut buf, fname)?;
    }

    Ok(())
}
