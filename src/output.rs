use std::collections::HashMap;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use csv::Writer;

use crate::donation::DonationUtils;
use crate::error::Error;
use crate::player::Player;

const SECS_IN_SEASON: u64 = 3600 * 24 * 28;
const ONE_RESET_SEASON: u64 = 1622433600;
const END_OF_SEASON_STARTING: u64 = 3600 * 24 * 5;

pub fn generate_csv<P>(path: P, players: &[Player]) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let donations = players
        .iter()
        .filter(|p| p.donations > 100 * !is_season_starting() as u32)
        .map(|p| &p.donable)
        .flatten()
        .collect::<Vec<_>>();

    let max_level = donations.iter().map(|d| d.max_level).max().unwrap_or(0);

    let mut sheet = HashMap::new();

    for donation in donations {
        let lvls = sheet
            .entry(&donation.name)
            .or_insert_with(|| vec![0u8; max_level + 1]);
        lvls[max_level - donation.level] += 1;
        lvls[max_level] += 1;
    }

    let mut sheet = sheet.into_iter().collect::<Vec<_>>();
    sheet.sort_by_key(|don| don.0.as_u32());

    if !PathBuf::from("./output").is_dir() {
        create_dir("./output")?;
    }

    let mut category = 0;
    let mut wtr = Writer::from_path(path)?;

    wtr.write_record(generate_colums(max_level))?;
    for (name, lvls) in sheet {
        if name.is_pet() {
            continue;
        }
        let id = name.as_u32();
        let donation_category = id / 100;
        if donation_category != category {
            wtr.write_record(vec![""; max_level + 2])?;
            category = donation_category;
        }
        wtr.write_field(name)?;
        wtr.write_record(lvls.iter().map(|lvl| {
            if lvl == &0 {
                String::new()
            } else {
                lvl.to_string()
            }
        }))?;
    }
    wtr.flush()?;

    Ok(())
}

#[inline]
fn generate_colums(max_level: usize) -> Vec<String> {
    let mut row = Vec::with_capacity(max_level + 2);
    row.push("DONATION".to_owned());
    row.append(
        &mut (1..=max_level)
            .rev()
            .map(|x| format!("LvL {}", x))
            .collect::<Vec<String>>(),
    );
    row.push("TOTAL".to_owned());
    row
}

#[inline]
fn is_season_starting() -> bool {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - ONE_RESET_SEASON)
        % SECS_IN_SEASON
        < END_OF_SEASON_STARTING
}
