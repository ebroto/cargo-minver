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
        augmented_assignments, //
        Edition::Edition2015,
        "1.8.0",
        ["11:4 11:11"]
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
        question_mark, //
        Edition::Edition2015,
        "1.13.0",
        ["8:14 8:20"]
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
        field_init_shorthand,
        Edition::Edition2015,
        "1.17.0",
        [
            "14:16 14:21", //
            "14:23 14:29",
            "14:31 14:36"
        ]
    ),
    (
        static_in_const,
        Edition::Edition2015,
        "1.17.0",
        [
            "3:12 3:16", //
            "6:11 6:15"
        ]
    ),
    (
        windows_subsystem, //
        Edition::Edition2015,
        "1.18.0",
        ["1:0 1:33"]
    ),
    (
        pub_restricted,
        Edition::Edition2015,
        "1.18.0",
        [
            "3:0 3:10", //
            "5:0 5:9",
            "8:4 8:13",
            "11:4 11:13",
            "14:4 14:13"
        ]
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
            "33:8 33:17",
            "39:13 39:28",
            "40:13 40:26",
            "42:8 42:23",
            "43:8 43:20",
            "44:8 44:21",
            "45:8 45:20"
        ]
    ),
    (
        struct_field_attributes,
        Edition::Edition2015,
        "1.20.0",
        [
            "16:8 16:17", //
            "20:8 20:17",
            "25:8 25:13",
            "29:8 29:13"
        ]
    ),
    (
        associated_consts,
        Edition::Edition2015,
        "1.20.0",
        [
            "4:4 4:17", //
            "9:4 9:22",
            "13:4 13:22"
        ]
    ),
    (
        abi_sysv64,
        Edition::Edition2015,
        "1.24.0",
        [
            "3:7 3:15", //
            "5:7 5:15"
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
        use_nested_groups,
        Edition::Edition2015,
        "1.25.0",
        [
            "4:22 4:23", //
            "5:16 5:32",
            "6:16 6:47"
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
        termination_trait, //
        Edition::Edition2015,
        "1.26.0",
        ["3:13 3:30"]
    ),
    (
        underscore_lifetimes,
        Edition::Edition2015,
        "1.26.0",
        [
            "10:12 10:14", //
            "16:23 16:25",
            "20:37 20:39",
            "24:38 24:40"
        ]
    ),
    (
        const_indexing,
        Edition::Edition2015,
        "1.26.0",
        [
            "2:20 2:30", //
            "5:17 5:27"
        ]
    ),
    (
        universal_impl_trait, //
        Edition::Edition2015,
        "1.26.0",
        ["5:7 5:17"]
    ),
    (
        conservative_impl_trait, //
        Edition::Edition2015,
        "1.26.0",
        ["6:12 6:18"]
    ),
    (
        min_slice_patterns,
        Edition::Edition2015,
        "1.26.0",
        [
            "5:8 5:23", //
            "6:8 6:23"
        ]
    ),
    (
        match_default_bindings, //
        Edition::Edition2015,
        "1.26.0",
        ["4:8 4:21"]
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
        dyn_trait, //
        Edition::Edition2015,
        "1.27.0",
        ["5:11 5:16"]
    ),
    (
        fn_must_use,
        Edition::Edition2015,
        "1.27.0",
        [
            "2:0 4:1", //
            "13:4 15:5"
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
        tool_attributes, //
        Edition::Edition2015,
        "1.30.0",
        ["3:0 3:16"]
    ),
    (
        pattern_parentheses, //
        Edition::Edition2015,
        "1.31.0",
        ["5:8 5:12"]
    ),
    (
        min_const_fn,
        Edition::Edition2015,
        "1.31.0",
        [
            "3:0 3:5", //
            "7:4 7:9"
        ]
    ),
    (
        impl_header_lifetime_elision,
        Edition::Edition2015,
        "1.31.0",
        [
            "6:7 6:9", //
            "9:13 9:15",
            "10:12 10:13",
            "10:18 10:19"
        ]
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
        irrefutable_let_patterns,
        Edition::Edition2015,
        "1.33.0",
        [
            "4:11 4:12", //
            "5:14 5:15",
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
        min_const_unsafe_fn,
        Edition::Edition2015,
        "1.33.0",
        [
            "15:13 15:18", //
            "19:28 19:33",
            "24:4 24:9",
            "30:17 30:22",
            "34:8 34:13",
            "38:17 38:22",
            "42:8 42:13"
        ]
    ),
    (
        extern_crate_self, //
        Edition::Edition2015,
        "1.34.0",
        ["1:0 1:25"]
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
        underscore_const_names, //
        Edition::Edition2015,
        "1.37.0",
        ["1:6 1:7"]
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
        bind_by_move_pattern_guards, //
        Edition::Edition2015,
        "1.39.0",
        ["5:8 5:12"]
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
        const_constructor,
        Edition::Edition2015,
        "1.40.0",
        [
            "9:13 9:18", //
            "10:13 10:21",
            "13:12 13:17",
            "14:12 14:20"
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
    ),
    (
        slice_patterns,
        Edition::Edition2015,
        "1.42.0",
        [
            "6:15 6:17", //
            "7:12 7:14",
            "8:9 8:11",
            "14:9 14:16",
            "15:12 15:19",
            "16:15 16:22"
        ]
    )
);
