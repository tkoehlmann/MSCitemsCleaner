/*********************************************
* See LICENSE file for licensing information *
*********************************************/

use std::io::Write;



// An entry from the items.txt
struct Entry {
    tag: String, // the tag name
    data: Vec<u8> // the binary data saved for this tag
}



// Receives a Result and an error message and calls 'exit' in case of an error
fn exit_on_error<T>(r: std::io::Result<T>, error_msg: &str) -> T {
    if r.is_err() {
        exit(error_msg)
    }
    return r.ok().unwrap();
}



// Prints an error message and quits the program
fn exit(msg: &str) {
    println!("{}", msg);
    
    #[cfg(target_os = "windows")]
    {
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    }

    std::process::exit(-1);
}



// Reads a little-endian u32 from the provided array
fn get_u32_le(buf: &Vec<u8>, idx: &mut usize) -> u32 {
    let res: u32 =
    ((buf[*idx + 0] as u32) <<  0) |
    ((buf[*idx + 1] as u32) <<  8) |
    ((buf[*idx + 2] as u32) << 16) |
    ((buf[*idx + 3] as u32) << 24) ;
    *idx += 4;
    res
}



// Creates a little-endian byte-vector from the provided number
fn mk_u32_le(n: &usize) -> Vec<u8> {
    vec![
        (n >>  0 & 0xFF) as u8,
        (n >>  8 & 0xFF) as u8,
        (n >> 16 & 0xFF) as u8,
        (n >> 24 & 0xFF) as u8
    ]
}



// Reads a string with a given length from the provided array
// the given index will be incremented by the amount of bytes read for
// convenience
fn get_string(buf: &Vec<u8>, idx: &mut usize, len: u32) -> String {
    let mut res = String::new();
    let mut i = 0;
    while i < len {
        res.push(buf[*idx + (i as usize)] as char);
        i += 1;
    }
    *idx += len as usize;
    res
}



// Expects the data from items.txt and generates the entries from it
fn generate_entries(file_contents: Vec<u8>) -> Vec<Entry> {
    let mut result = Vec::new();
    let mut i: usize = 0;
    
    while i < file_contents.len() {
        let mut new_entry: Entry = Entry { tag: String::new(), data: Vec::new() };

        // check entry header
        if file_contents[i] != 0x7E {
            exit(format!("Invalid header symbol at position {:#10x}", i).as_str());
        }
        i += 1;

        // read tag name
        let tag_length = file_contents[i];
        i += 1;
        new_entry.tag = get_string(&file_contents, &mut i, tag_length as u32);

        // read data
        let data_length = get_u32_le(&file_contents, &mut i);
        for j in 0..data_length-1 { // -1 because the final byte is the footer
            new_entry.data.push(file_contents[i + j as usize]);
        }
        i += (data_length - 1) as usize;

        // check entry footer
        if file_contents[i] != 0x7B {
            exit(format!("Invalid footer symbol at position {:#10x}", i).as_str());
        }
        i += 1;

        result.push(new_entry);
    }
    result
}



// Saves the entries into items.txt, overwriting it (make sure to call 'backup_items_file' first)
fn save_new_items_file(entries: &Vec<Entry>) {
    let items_file_path = std::path::PathBuf::from("items.txt");
    exit_on_error(std::fs::remove_file(&items_file_path), "Failed to delete \"items.txt\"");
    let items_file = exit_on_error(std::fs::File::create(&items_file_path), "Failed to create \"items.txt\"");
    let mut writer = std::io::BufWriter::new(items_file);

    for e in entries {
        // header
        exit_on_error(writer.write(&[0x7E as u8]), "I/O error while writing to \"items.txt\"");
        // tag name
        exit_on_error(writer.write(&[e.tag.len() as u8]), "I/O error while writing to \"items.txt\"");
        exit_on_error(writer.write(&e.tag.as_bytes()), "I/O error while writing to \"items.txt\"");
        // data
        exit_on_error(writer.write(&mk_u32_le(&(&e.data.len() + 1))), "I/O error while writing to \"items.txt\""); // + 1 because the length includes the footer
        exit_on_error(writer.write(&e.data), "I/O error while writing to \"items.txt\"");
        // footer
        exit_on_error(writer.write(&[0x7B as u8]), "I/O error while writing to \"items.txt\"");
    }

    exit_on_error(writer.flush(), "I/O error while writing to \"items.txt\"");
}



// Checks whether an item is located at the dedicated landfill position
fn is_in_landfill(entry: &Entry) -> bool {
    // Example:
    //      Landfill position:     -679.3277587891, 4.5722312927, -727.2958374023 (determined with MSC Editor)
    //      pikex36Transform data: FF 76 FA 7A 09 04 FA D4 29 C4 B8 4F 92 40 EF D2 35 C4 A1 1F 6B 3D 9C 62 EF 3E BB F8 80 3D E0 3D 61 BF 00 00 80 3F 01 00 80 3F 01 00 80 3F 08 55 6E 74 61 67 67 65 64
    //                         X:                    |---------|
    //                         Y:                                |---------|
    //                         Z:                                            |---------|
    let landfill_pos: [u8; 12] = [0xFA, 0xD4, 0x29, 0xC4, 0xB8, 0x4F, 0x92, 0x40, 0xEF, 0xD2, 0x35, 0xC4];

    if entry.data.len() < 18 {
        return false
    }

    let mut same = true;
    for i in 6..18 { // Thanks Rust, really intuitive to write "6..18" when I want to run up to 17
        same &= entry.data[i] == landfill_pos[i-6];
    }
    same
}



// Trims the item id from a full tag name, i.e. "pikex36Transform" -> "pikex36"
fn get_item_id(tag: &String) -> String {
    let mut numbers_count = 0;
    let tag_array = tag.as_bytes();
    for i in 0..tag_array.len() {
        let c = tag_array[i];
        if numbers_count == 0 {
            if c >= '0' as u8 && c <= '9' as u8 {
                numbers_count += 1
            }
        } else {
            if 
                c < '0' as u8 || c > '9' as u8 ||
                (
                    tag.starts_with("spraycan") &&
                    numbers_count >= 2
                )
            {
                return String::from_utf8(tag_array[0..i].to_vec()).unwrap();
            } else {
                numbers_count += 1
            }
        }
    }
    String::from(tag)
}



// Sets the "count" of a tag to a new one
// (i.e. "sausagesx11Transform" -> "sausagesx7Transform")
fn tag_set_new_count(e: &mut Entry, n: usize) {
    let tag_clone = e.tag.clone();
    let tag_array = tag_clone.as_bytes();

    let mut start_num_pos = 0;
    let mut end_num_pos = 0;

    for i in 0..tag_array.len() {
        let c = tag_array[i];
        if c >= '0' as u8 && c <= '9' as u8 {
            if start_num_pos == 0 {
                start_num_pos = i
            }
        } else if start_num_pos > 0 && end_num_pos == 0 {
            end_num_pos = i
        }
    }

    e.tag = format!("{}{}{}",
        String::from_utf8(tag_array[0..start_num_pos].to_vec()).unwrap(),
        n,
        String::from_utf8(tag_array[end_num_pos..tag_array.len()].to_vec()).unwrap()
    );
}



// Removes unwanted entries from the provided ones
fn clean_entries(entries: Vec<Entry>) -> Vec<Entry> {
    
    // In this initial version of the program we'll simply delete all items that
    // are in the dedicated landfill spot. This is probably the safest thing to
    // do, even if it won't fully "clear" the save of all used up items.

    // These are present on a fresh save game and if touched weird things happen,
    // probably because TG hardcoded some stuff. So we won't touch those entries.

    // TODO: Verify whether this is still true after getting everything to work
    //       properly!
    let dont_touch_entries = vec![
        "milkxTransform",
        "milkxCondition",
        "sausagesx0",
        "pizzaxTransform",
        "pizzaxCondition",
        "beercase0",
        "macaron boxxTransform",
        "macaron boxxCondition",
        "oilfilter0",
    ];

    // A blacklist of tag-beginnings that can end up in the landfill but can be
    // attached to the car, the house, or the radio so (for now) we'll not touch
    // these as they surely are referenced by ID somewhere else and changing IDs
    // might cause some save file or game corruption if not handled properly
    let blacklist = vec![
        "fireextinguisher",
        "n2obottle", // ID correct?
        "battery",
        "oil filter,",
        "spark plug",
        "alternator belt",
        "light bulb",
        "fuse",
        "r20 battery"
    ];

    // determine the items that are in the landfill
    let mut located_in_landfill: Vec<String> = Vec::new();
    for e in &entries {
        let itemid = get_item_id(&e.tag);

        if
            is_in_landfill(e) &&
            !dont_touch_entries.contains(&itemid.as_str()) &&
            blacklist.iter().all(|bi| !e.tag.starts_with(bi))
        {
            located_in_landfill.push(itemid);
        }
    }

    // push everything that's not in the landfill into the result vector
    let mut res: Vec<Entry> = Vec::new();
    for e in entries {
        let itemid = get_item_id(&e.tag);
        if !located_in_landfill.contains(&itemid) {
            res.push(e)
        }
    }

    /*
     * In future versions we also want to remove all items that are not in the
     * landfill but are still consumed. For some items this can be checked with
     * the "...Consumed" entry, but it's more complicated for others
     * (i.e. pikes only have a "Condition" variable)
     */
    
    
    // recount and set all entry IDs so that they start at 1, except for the special cases
    struct Group {
        tagname: String,
        tagid: String,
        has_default_zero_item: bool,
        count: usize,
        max: usize
    }
    let mut item_counts: Vec<Group> = vec![
        Group { tagname: String::from("beercase"), tagid: String::from("BeerCaseID"), has_default_zero_item: true, count: 0, max: 0 },
        Group { tagname: String::from("sausagesx"), tagid: String::from("SausagesxID"), has_default_zero_item: true, count: 0, max: 0 },
        Group { tagname: String::from("milkx"), tagid: String::from("milkxID"), has_default_zero_item: true, count: 0, max: 0 },
        Group { tagname: String::from("sugar"), tagid: String::from("sugarID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("yeast"), tagid: String::from("yeastID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("potatochips"), tagid: String::from("potatochipsID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("pizzax"), tagid: String::from("pizzaxID"), has_default_zero_item: true, count: 0, max: 0 },
        Group { tagname: String::from("macaronbox"), tagid: String::from("macaronboxxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("shoppingbagx"), tagid: String::from("shoppingbagxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("moosemeatx"), tagid: String::from("moosemeatxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("Booze"), tagid: String::from("BoozeID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("pikex"), tagid: String::from("pikexID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("juiceconcentrate"), tagid: String::from("juiceconcentrateID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("motoroil"), tagid: String::from("motoroilID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("brakefluid"), tagid: String::from("brakefluidID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("coolant"), tagid: String::from("coolantID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("twostroke"), tagid: String::from("twostrokeID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("cigarettes"), tagid: String::from("cigarettesID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spark plug box"), tagid: String::from("sparkplugboxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("groundcoffee"), tagid: String::from("groundcoffeeID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("grillcharcoal"), tagid: String::from("grillcharcoalID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("light bulb box"), tagid: String::from("lightbulbboxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("fuse package"), tagid: String::from("fusepackageID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("r20 battery box"), tagid: String::from("r20batteryboxID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("mosquitospray"), tagid: String::from("mosquitosprayID"), has_default_zero_item: false, count: 0, max: 0 },
        // Spraycans are untested, here be dragons        
        Group { tagname: String::from("spraycan01"), tagid: String::from("Spraycan01ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan02"), tagid: String::from("Spraycan02ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan03"), tagid: String::from("Spraycan03ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan04"), tagid: String::from("Spraycan04ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan05"), tagid: String::from("Spraycan05ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan06"), tagid: String::from("Spraycan06ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan07"), tagid: String::from("Spraycan07ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan08"), tagid: String::from("Spraycan08ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan09"), tagid: String::from("Spraycan09ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan10"), tagid: String::from("Spraycan10ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan11"), tagid: String::from("Spraycan11ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan12"), tagid: String::from("Spraycan12ID"), has_default_zero_item: false, count: 0, max: 0 },
        Group { tagname: String::from("spraycan13"), tagid: String::from("Spraycan13ID"), has_default_zero_item: false, count: 0, max: 0 },
    ];

    // These items here won't yet be touched, see the comment on the variable
    // 'blacklist' for an explanation

    // fireextinguisher, fireextinguisherID
    // battery, batteryID
    // oil filter, oilfilterID
    // spark plug, sparkplugID
    // alternator belt, alternatorbeltID
    // light bulb, lightbulbID
    // fuse, fuseID -> most likely fuseholderXX entries in items.txt
    // r20 battery, r20batteryID

    // Checking the defaultES2file.txt suggests that most stuff that's mounted
    // into/on the car gets removed from items.txt and gets moved there, but
    // further checks are needed before implementing it.
    
    
    // Count items, modify entry counters
    for e in &mut res {
        for g in &mut item_counts {
            // We check for Transform here because a) it always exists, and
            // b) because otherwise we'd be counting things like
            // "yeast12Transform" and "yeast12Consumed" twice
            if e.tag.starts_with(&g.tagname) && e.tag.ends_with("Transform") {
                g.count += 1;
                g.max += 1;
            }
        }
    }

    // Recude counters by 1 (where possible) because the field holds the highest
    // item group ID, not the count
    for g in &mut item_counts {
        if g.count >= 1 {
            if g.has_default_zero_item {
                g.count -= 1;
                g.max -= 1
            }
        }
    }

    #[derive(Clone)]
    struct Map {
        oldid: String,
        newid: String
    }
    let mut map: Vec<Map> = Vec::new();

    // rename items
    for e in &mut res {
        if
            dont_touch_entries.contains(&e.tag.as_str()) ||
            blacklist.iter().any(|bi| e.tag.starts_with(bi))
        {
            continue;
        }

        for g in &mut item_counts {
            if g.tagid == e.tag {
                continue;
            }

            if e.tag.starts_with(&g.tagname) {
                // Look up tag in map
                let id = get_item_id(&e.tag);
                match map.iter().find(|&e| e.oldid == id) {
                    Some(m) => {
                        // if found then we just replace the tag with the mapped one
                        let mapped_item = m.clone();
                        e.tag = e.tag.replace(&mapped_item.oldid, &mapped_item.newid)
                    },
                    None => {
                        // otherwise we add it to the map with the new counter
                        tag_set_new_count(e, g.count);
                        if g.count > 0 {
                            g.count -= 1;
                        }
                        map.push(Map { oldid: id, newid: get_item_id(&e.tag) });
                    }
                };
            }
        }
    }

    // finally: find BeerCaseID, SausagesxID, milkxID, sugarID, yeastID,
    //          potatochipsID, pizzaxID, macaronboxID, shoppingbagxID,
    //          moosemeatxID, BoozeID, pikexID (and maybe some others in the
    //          future) and set their IDs to the highest ID of the corresponding
    //          item group

    // TODO: In the original file the IDs are descending. Is this a requirement?

    for e in &mut res {
        for g in &item_counts {
            if e.tag == g.tagid {
                let count = mk_u32_le(&g.max);
                /*
                 * BeerCaseID:  FF 56 08 A8 E2 (0A 00 00 00)
                 * SausagesxID: FF 56 08 A8 E2 (36 00 00 00)
                 * ...
                 */
                for pos in 5..9 {
                    e.data[pos] = count[pos - 5];
                }
            }
        }
    }

    res
}



// Creates a safety-save of the items.txt
fn backup_items_file() {
    // creates a filepath with the given number in it
    fn fnamep(i: usize) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("items{:0>2}.txt", i))
    }

    // how many backups should be held
    let max_backup_counter: usize = 10;
    {
        let p = fnamep(max_backup_counter);
        if std::path::Path::is_file(&p) {
            exit_on_error(std::fs::remove_file(&p), format!("Failed to remove file \"{}\"", p.display()).as_str())
        }
    }
    for i in (0..10).rev() { // Rust... just why. Was 10..0 (or 9..-1 I guess) really that syntactically complex?
        let from = fnamep(i);
        let to = fnamep(i + 1);
        if std::path::Path::is_file(&from) {
            exit_on_error(std::fs::rename(&from, &to), format!("Failed to rename \"{}\"", from.display()).as_str());
        }
    }

    exit_on_error(std::fs::copy("items.txt", "items00.txt"), "Failed to rename \"items.txt\"");
}



// Generates a vector of strings describing all entries (and also the counter for the counting tags)
#[cfg(debug_assertions)]
fn get_formatted_entries(entries: &Vec<Entry>) -> Vec<String> {
    let counting_tags = vec![
        "BeerCaseID", "SausagesxID", "milkxID", "sugarID", "yeastID",
        "potatochipsID", "pizzaxID", "macaronboxxID", "shoppingbagxID",
        "moosemeatxID", "BoozeID", "pikexID", "juiceconcentrateID",
        "motoroilID", "brakefluidID", "coolantID", "twostrokeID",
        "cigarettesID", "sparkplugboxID", "groundcoffeeID", "grillcharcoalID",
        "lightbulbboxID", "fusepackageID", "r20batteryboxID", "mosquitosprayID"
    ];

    let mut res: Vec<String> = Vec::new();
    for e in entries {
        if counting_tags.contains(&e.tag.as_str()) {
            let mut idx: usize = 5;
            res.push(format!("{} ({})", e.tag, get_u32_le(&e.data, &mut idx)));
        } else {
            res.push(format!("{}", e.tag));
        }
    }
    res
}



// Saves the list of entries to a file
#[cfg(debug_assertions)]
fn save_entries_list(entries: &Vec<Entry>) {
    let fmt = get_formatted_entries(&entries);
    let mut out = String::new();
    for e in fmt {
        out.push_str(format!("{}{}", if out == "" { "" } else { "\n" },  e).as_str());
    }
    exit_on_error(std::fs::write("items_list.txt", out), "Failed saving \"items_list.txt\"")
}



fn main() {
    let items_file: Vec<u8> = exit_on_error(
        std::fs::read("items.txt"),
        "File \"items.txt\" was not found or couldn't be read! Make sure the executable is in the same folder as the file."
    );

    backup_items_file();
    let mut entries: Vec<Entry> = generate_entries(items_file);
    entries = clean_entries(entries);
    save_new_items_file(&entries);
    
    #[cfg(debug_assertions)]
    save_entries_list(&entries)
}
