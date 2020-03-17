#[macro_use]
mod util;

use util::project::Edition;

// Each entry creates a test for a language feature. Input files are taken from the lang_files directory.
// The format is: (feature name, edition, rustc version, spans, inspect?). Inspect is just for debugging.
test_lang_features!(
    (
        braced_empty_structs,
        Edition::Edition2015,
        "1.8.0",
        [
            "3:0 3:11", //
            "13:4 13:8",
            "20:12 20:16",
            "23:8 23:12",
            "27:12 27:21",
            "30:8 30:16",
            "34:12 34:19",
            "37:8 37:15",
            "42:8 42:19",
        ]
    ),
    (
        deprecated, //
        Edition::Edition2015,
        "1.9.0",
        ["3:0 3:13"]
    ),
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
        repr_align,
        Edition::Edition2015,
        "1.25.0",
        [
            "4:0 6:1", //
            "9:0 12:1"
        ],
        true
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
        cfg_target_feature,
        Edition::Edition2015,
        "1.27.0",
        [
            "3:6 3:26", //
            "6:10 6:30",
            "10:12 10:32",
            "11:16 11:36"
        ]
    ),
    (
        target_feature, //
        Edition::Edition2015,
        "1.27.0",
        ["3:0 3:34"]
    ),
    (
        repr_transparent,
        Edition::Edition2015,
        "1.28.0",
        [
            "6:0 6:14", //
            "9:0 11:1",
            "14:0 17:1",
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
        used, //
        Edition::Edition2015,
        "1.30.0",
        ["2:0 2:21"]
    ),
    (
        pattern_parentheses,
        Edition::Edition2015, //
        "1.31.0",
        ["5:8 5:12"]
    ),
    (
        if_while_or_patterns,
        Edition::Edition2015,
        "1.33.0",
        [
            "2:11 2:18", //
            "3:14 3:21",
        ]
    ),
    (
        underscore_imports,
        Edition::Edition2015,
        "1.33.0",
        [
            "3:20 3:21", //
            "4:16 4:17",
        ]
    ),
    (
        repr_packed, //
        Edition::Edition2015,
        "1.33.0",
        ["4:0 6:1"]
    ),
    (
        cfg_target_vendor,
        Edition::Edition2015,
        "1.33.0",
        [
            "3:6 3:31", //
            "6:10 6:35",
            "10:12 10:37",
            "11:16 11:41"
        ]
    ),
    (
        cfg_attr_multi,
        Edition::Edition2015,
        "1.33.0",
        [
            "3:0 3:69", //
            "7:0 7:20",
            "10:18 10:84"
        ]
    ),
    (
        repr_align_enum, //
        Edition::Edition2015,
        "1.37.0",
        ["4:0 6:1"]
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
    ),
    (
        param_attrs,
        Edition::Edition2015,
        "1.39.0",
        [
            "3:8 3:42", //
            "7:51 7:56",
            "11:28 11:62",
            "14:28 14:62",
            "18:34 18:64",
            "21:14 21:43",
            "22:14 22:48"
        ]
    ),
    (
        non_exhaustive,
        Edition::Edition2015,
        "1.40.0",
        [
            "4:0 4:11", //
            "7:0 7:9",
        ]
    ),
    (
        cfg_doctest,
        Edition::Edition2015,
        "1.40.0",
        [
            "3:6 3:13", //
            "6:10 6:17",
            "10:12 10:19",
            "11:16 11:23"
        ]
    ),
    (
        transparent_enums,
        Edition::Edition2015,
        "1.42.0",
        [
            "4:0 6:1", //
            "9:0 11:1",
            "14:0 16:1",
        ]
    )
);
