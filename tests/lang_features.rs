#[macro_use]
mod util;

use util::project::Edition;

// Each entry creates a test for a language feature. Input files are taken from the lang_files directory.
// The format is: (feature name, edition, rustc version, spans)
test_lang_features!(
    (
        dotdot_in_tuple_patterns,
        Edition::Edition2015,
        "1.14.0",
        [
            "4:8 4:20", //
            "5:8 5:20",
            "6:8 6:20",
            "13:8 13:24",
            "14:8 14:24",
            "15:8 15:24"
        ]
    ),
    (
        loop_break_value, //
        Edition::Edition2015,
        "1.19.0",
        ["3:8 3:16"]
    ),
    (
        dotdoteq_in_patterns, //
        Edition::Edition2015,
        "1.26.0",
        ["3:8 3:14"]
    ),
    (
        inclusive_range_syntax, //
        Edition::Edition2015,
        "1.26.0",
        ["2:18 2:23"]
    ),
    (
        i128_type,
        Edition::Edition2015,
        "1.26.0",
        [
            "1:13 1:17",
            "1:22 1:26",
            "5:13 5:17",
            "5:22 5:26",
            "10:12 10:16",
            "11:12 11:16",
            "12:13 12:19",
            "13:13 13:19"
        ]
    ),
    (
        crate_in_paths,
        Edition::Edition2015,
        "1.30.0",
        [
            "12:4 12:9",
            "14:12 14:17",
            "16:8 16:13",
            "20:12 20:17",
            "21:11 21:16",
            "25:8 25:13",
            "27:9 27:14",
            "35:26 35:31",
            "35:45 35:50"
        ]
    ),
    (
        raw_identifiers,
        Edition::Edition2015,
        "1.30.0",
        [
            "2:8 2:15", //
            "3:13 3:20",
        ]
    ),
    (
        async_await,
        Edition::Edition2018,
        "1.39.0",
        [
            "4:4 4:29", //
            "7:0 12:1",
            "8:17 8:25",
            "11:4 11:20"
        ]
    )
);
