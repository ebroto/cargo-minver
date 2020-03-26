#[macro_use]
mod util;

use util::project::{Edition, PanicBehavior};

// Each entry creates a test for a language feature. Input files are taken from the lang_files directory.
// The format is: (feature name, edition, rustc version, spans, on_panic?, inspect?). Inspect is just for debugging.
test_lang_features!(
    (
        no_std, //
        Edition::Edition2015,
        "1.6.0",
        ["1:0 1:10"]
    ),
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
        more_struct_aliases,
        Edition::Edition2015,
        "1.16.0",
        [
            "7:16 7:23", //
            "10:12 10:19",
            "20:12 20:19",
            "23:8 23:15"
        ]
    ),
    (
        windows_subsystem, //
        Edition::Edition2015,
        "1.18.0",
        ["1:0 1:33"]
    ),
    (
        loop_break_value, //
        Edition::Edition2015,
        "1.19.0",
        ["3:8 3:16"]
    ),
    (
        relaxed_adts,
        Edition::Edition2015,
        "1.19.0",
        [
            "4:0 4:17", //
            "8:4 8:13",
            "23:13 23:24",
            "24:13 24:25",
            "26:8 26:20",
            "27:8 27:17",
            "30:14 30:24",
            "32:8 32:18",
            "38:13 38:28",
            "39:13 39:26",
            "41:8 41:23",
            "42:8 42:20",
            "43:8 43:21"
        ]
    ),
    (
        struct_field_attributes,
        Edition::Edition2015,
        "1.20.0",
        [
            "14:12 17:5", //
            "18:12 21:5",
            "23:8 26:5",
            "27:8 30:5"
        ]
    ),
    (
        repr_align,
        Edition::Edition2015,
        "1.25.0",
        [
            "4:0 6:1", //
            "9:0 12:1"
        ]
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
        generic_param_attrs,
        Edition::Edition2015,
        "1.27.0",
        [
            "3:16 3:29",
            "4:16 4:29",
            "5:5 5:18",
            "6:5 6:18",
            "8:12 8:25",
            "11:12 11:25",
            "15:14 15:27",
            "18:14 18:27",
            "21:5 21:18",
            "26:5 26:18",
            "26:22 26:35",
            "30:10 30:23",
            "33:10 33:23",
            "36:12 36:25",
            "40:12 40:25",
            "43:12 43:25",
            "43:30 43:43",
            "44:12 44:25",
            "48:23 48:36"
        ]
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
        panic_handler, //
        Edition::Edition2015,
        "1.30.0",
        ["5:0 5:16"],
        PanicBehavior::Abort
    ),
    (
        pattern_parentheses,
        Edition::Edition2015, //
        "1.31.0",
        ["5:8 5:12"]
    ),
    (
        self_struct_ctor,
        Edition::Edition2015,
        "1.32.0",
        [
            "7:8 7:12", //
            "12:12 12:20",
            "18:16 18:20",
            "26:8 26:12",
            "31:12 31:16"
        ]
    ),
    (
        self_in_typedefs,
        Edition::Edition2015,
        "1.32.0",
        [
            "8:4 8:8", //
            "9:17 9:21",
            "11:12 11:16",
            "17:4 17:8",
            "18:17 18:21",
            "20:13 20:17",
            "26:4 26:8",
            "27:17 27:21",
            "29:13 29:17",
            "35:4 35:8",
            "36:17 36:21",
            "38:13 38:17"
        ]
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
        type_alias_enum_variants, //
        Edition::Edition2015,
        "1.37.0",
        [
            "11:16 11:40",
            "12:16 12:27",
            "13:16 13:26",
            "16:12 16:31",
            "17:12 17:27",
            "18:12 18:22",
            "26:12 26:33",
            "27:12 27:20",
            "28:12 28:19",
            "31:8 31:24",
            "32:8 32:20",
            "33:8 33:15"
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
            "7:38 7:43",
            "11:28 11:62",
            "14:28 14:62",
            "18:34 18:65",
            "22:21 22:55",
            "29:14 29:43",
            "30:14 30:48"
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
