/*
the base logic of how would the user want to name the files for the MoveFiles
for now it only by time
*/

use chrono::Utc;

pub fn create_naming_by_time(name: &str, extension: &str) -> String {
    let utc = Utc::now();
    let time_utc = format!("{}", utc);

    let time_splitted_vec = time_utc.split_whitespace().collect::<Vec<&str>>();
    let mut time_spiltted = time_splitted_vec[0].split('-').collect::<Vec<&str>>();
    time_spiltted.reverse();
    let time = time_spiltted.join("");

    format!("{name}{time}.{extension}")
}

#[cfg(test)]
mod tests {
    use crate::tools::file_naming::*;

    #[test]
    fn check_how_time_is_created() {
        let time = create_naming_by_time("צפי הגעת מוצרים", "csv");
        println!("{:?}", time);
    }
}
