//!
//! Benchmark tests for SMT parsing and satisfiability
//!

use super::parse::{parse_smtlib_file, SmtParser};
use super::util::hex_to_char;

use crate::solver::{satisfiable_all, satisfiable_default, NUM_SOLVERS};
use crate::types::expr::{CharExpression, StringVar};
use crate::types::regex::GenRegex;

use std::rc::Rc;

/*
    Helper functions for running tests
*/

// Assert that satisfiable() on the GenRegex returns as expected

fn assert_regex_helper(gre: &Rc<GenRegex>, expected: bool, default_only: bool) {
    if default_only {
        let result = satisfiable_default(gre);
        assert_eq!(result, expected);
    } else {
        let results = satisfiable_all(gre);
        assert!(results.len() == NUM_SOLVERS);
        assert!(results.iter().all(|&result| result == expected));
    }
}

// Run the SMT2 file and assert that satisfiable() returns as expected
fn assert_smt2_file_helper(filepath: &str, expected: bool, default_only: bool) {
    // Read the file and parse as s-expression
    let smt_result = parse_smtlib_file(filepath);
    println!("Parsed s-expression: {:?}", smt_result);
    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);
    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = Rc::new(gen_regex.unwrap());

    // Check result
    assert_regex_helper(&gen_regex_unwrapped, expected, default_only);
}

fn assert_satisfiable(filepath: &str) {
    assert_smt2_file_helper(filepath, true, false);
}
fn assert_unsatisfiable(filepath: &str) {
    assert_smt2_file_helper(filepath, false, false);
}
fn assert_satisfiable_regex(gre: &Rc<GenRegex>) {
    assert_regex_helper(gre, true, false);
}
fn assert_unsatisfiable_regex(gre: &Rc<GenRegex>) {
    assert_regex_helper(gre, false, false);
}

fn assert_satisfiable_default_only(filepath: &str) {
    assert_smt2_file_helper(filepath, true, true);
}
fn assert_unsatisfiable_default_only(filepath: &str) {
    assert_smt2_file_helper(filepath, false, true);
}
fn assert_satisfiable_regex_default_only(gre: &Rc<GenRegex>) {
    assert_regex_helper(gre, true, true);
}
fn assert_unsatisfiable_regex_default_only(gre: &Rc<GenRegex>) {
    assert_regex_helper(gre, false, true);
}

/*
    Tests - easiest cases
*/

#[test]
fn s_expr_test() {
    // Basic unit test for parsing SMTLib files
    // Note that we have to add the beginning '(' and ending ')' to the string
    // so that it makes a single S-expression.

    let smt_string = r#"
(
(set-logic QF_S)
;---
; .NET regular expressions restricted to 7-bit characters
; membership in intersection of
; .*(monday|tuesday|wednesday|thursday|friday|saturday|sunday).*
; .*(january|february|march|april|may|june|july|august|september|october|november|december).*
; [!-~]*
;---
(declare-const x String)
(assert (str.in_re x (re.inter (re.inter (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "monday") (str.to_re "tuesday")) (str.to_re "wednesday")) (str.to_re "thursday")) (str.to_re "friday")) (str.to_re "saturday")) (str.to_re "sunday"))) re.all) (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "january") (str.to_re "february")) (str.to_re "march")) (str.to_re "april")) (str.to_re "may")) (str.to_re "june")) (str.to_re "july")) (str.to_re "august")) (str.to_re "september")) (str.to_re "october")) (str.to_re "november")) (str.to_re "december"))) re.all)) (re.* (re.range "!" "~")))))
(check-sat)
;(get-model)
)
"#;

    println!("{}", smt_string);
    let v = lexpr::from_str(smt_string).unwrap();
    println!("{:?}", v);

    // Uncomment to view output
    // assert!(false);
}

#[test]
fn test_simple_1() {
    // Load the file simple1.smt2
    // Parse as s-expression
    let smt_result = parse_smtlib_file("benchmarks/simple1_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    // Expected output
    let expected = GenRegex::Intersect(
        Rc::new(GenRegex::StringVar(StringVar {
            name: "x".to_string(),
        })),
        Rc::new(GenRegex::Concatenation(
            Rc::new(GenRegex::CharExpression(CharExpression::Literal('a'))),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal('b'))),
        )),
    );

    assert_eq!(gen_regex_unwrapped, expected);

    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_simple_2() {
    let smt_result = parse_smtlib_file("benchmarks/simple2_unsat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    // Expected output
    let expected_str_var = GenRegex::StringVar(StringVar {
        name: "x".to_string(),
    });
    let expected_intersection_1 = GenRegex::Intersect(
        Rc::new(expected_str_var.clone()),
        Rc::new(GenRegex::CharExpression(CharExpression::Literal('a'))),
    );
    let expected_intersection_2 = GenRegex::Intersect(
        Rc::new(expected_str_var),
        Rc::new(GenRegex::CharExpression(CharExpression::Literal('b'))),
    );

    let expected = GenRegex::Concatenation(
        Rc::new(expected_intersection_1),
        Rc::new(expected_intersection_2),
    );

    assert_eq!(gen_regex_unwrapped, expected);

    assert_unsatisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_simple_3() {
    let smt_result = parse_smtlib_file("benchmarks/simple3_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    // Expected output
    let expected_str_var_x = GenRegex::StringVar(StringVar {
        name: "x".to_string(),
    });
    let expected_str_var_y = GenRegex::StringVar(StringVar {
        name: "y".to_string(),
    });
    let expected_intersection_1 = GenRegex::Intersect(
        Rc::new(expected_str_var_x),
        Rc::new(GenRegex::CharExpression(CharExpression::Literal('a'))),
    );
    let expected_intersection_2 = GenRegex::Intersect(
        Rc::new(expected_str_var_y),
        Rc::new(GenRegex::CharExpression(CharExpression::Literal('b'))),
    );

    let expected = GenRegex::Concatenation(
        Rc::new(expected_intersection_1),
        Rc::new(expected_intersection_2),
    );
    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));

    assert_eq!(gen_regex_unwrapped, expected);
}

#[test]
fn test_range() {
    let smt_result = parse_smtlib_file("benchmarks/range1_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let expected = GenRegex::Intersect(
        GenRegex::create_gre_str_var("x"),
        GenRegex::re_range('0', '9'),
    );

    assert_eq!(gen_regex_unwrapped, expected);

    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_re_all() {
    let smt_result = parse_smtlib_file("benchmarks/re_all_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let union = GenRegex::union(
        &GenRegex::create_gre_char_lit('a'),
        &GenRegex::create_gre_char_lit('b'),
    );
    let regex = GenRegex::concat(&GenRegex::sigma_star(), &union);
    let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

    assert_eq!(gen_regex_unwrapped, expected);

    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_date() {
    let smt_result = parse_smtlib_file("excluded/from_regexbenchmarks/date_sat.smt2");
    println!("Parsed s-expression: {:?}\n", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let dot_star = GenRegex::sigma_star();
    let mut days_of_the_week: Vec<Rc<GenRegex>> = Vec::new();
    days_of_the_week.push(GenRegex::str_to_re("monday"));
    days_of_the_week.push(GenRegex::str_to_re("tuesday"));
    days_of_the_week.push(GenRegex::str_to_re("wednesday"));
    days_of_the_week.push(GenRegex::str_to_re("thursday"));
    days_of_the_week.push(GenRegex::str_to_re("friday"));
    days_of_the_week.push(GenRegex::str_to_re("saturday"));
    days_of_the_week.push(GenRegex::str_to_re("sunday"));
    let mut union_days = days_of_the_week[0].clone();
    for v in &days_of_the_week[1..] {
        union_days = GenRegex::union(&union_days, v);
    }
    let mut months: Vec<Rc<GenRegex>> = Vec::new();
    months.push(GenRegex::str_to_re("january"));
    months.push(GenRegex::str_to_re("february"));
    months.push(GenRegex::str_to_re("march"));
    months.push(GenRegex::str_to_re("april"));
    months.push(GenRegex::str_to_re("may"));
    months.push(GenRegex::str_to_re("june"));
    months.push(GenRegex::str_to_re("july"));
    months.push(GenRegex::str_to_re("august"));
    months.push(GenRegex::str_to_re("september"));
    months.push(GenRegex::str_to_re("october"));
    months.push(GenRegex::str_to_re("november"));
    months.push(GenRegex::str_to_re("december"));
    let mut union_months = months[0].clone();
    for v in &months[1..] {
        union_months = GenRegex::union(&union_months, v);
    }
    let first = GenRegex::concat(
        &GenRegex::concat(&dot_star.clone(), &union_days),
        &dot_star.clone(),
    );
    let second = GenRegex::concat(
        &GenRegex::concat(&dot_star.clone(), &union_months),
        &dot_star.clone(),
    );
    let third = GenRegex::star(&GenRegex::re_range('!', '~'));
    let regex = GenRegex::intersect(&&GenRegex::intersect(&first, &second), &third);
    let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

    assert_eq!(gen_regex_unwrapped, expected);

    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_passw_sat1() {
    let smt_result = parse_smtlib_file("excluded/from_regexbenchmarks/passw_sat1.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let dot_star = GenRegex::sigma_star();
    let first = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('a', 'z')),
        &dot_star,
    );
    let second = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('A', 'Z')),
        &dot_star,
    );
    let third = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('0', '9')),
        &dot_star,
    );
    let fourth = GenRegex::re_loop(0, 3, &GenRegex::re_range('!', '~'));
    let regex = GenRegex::intersect(
        &GenRegex::intersect(&GenRegex::intersect(&first, &second), &third),
        &fourth,
    );
    let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

    assert_eq!(gen_regex_unwrapped, expected);
    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped));
}

#[test]
fn test_simple_hex() {
    println!("A number{:?}", hex_to_char(0));
    let smt_result = parse_smtlib_file("benchmarks/simplehex_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();
    let hex = hex_to_char(0x0).unwrap();
    let expected = GenRegex::Intersect(
        GenRegex::create_gre_str_var("x"),
        GenRegex::re_range(hex, '/'),
    );

    assert_eq!(gen_regex_unwrapped, expected);
    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

// TODO: debug
// Stopped working after adding Determinized deriv experiment
#[test]
#[ignore]
fn test_passw_complement_3() {
    assert_satisfiable("excluded/from_regexbenchmarks/passw_sat_4.smt2");
}

#[test]
fn test_reglan_var() {
    assert_satisfiable("benchmarks/reglan_var_test_sat.smt2");
}

#[test]
fn test_let_1() {
    assert_satisfiable("benchmarks/simple_let_sat_1.smt2");
}

#[test]
fn test_let_2() {
    assert_satisfiable("benchmarks/simple_let_sat_2.smt2");
}

#[test]
fn test_let_3() {
    assert_satisfiable("benchmarks/simple_let_sat_3.smt2");
}

#[test]
fn test_let_4() {
    assert_satisfiable("benchmarks/simple_let_sat_4.smt2");
}

#[test]
fn test_define_fun1() {
    assert_satisfiable("benchmarks/simple_definefun_sat_1.smt2");
}

#[test]
fn test_define_fun2() {
    assert_satisfiable("benchmarks/simple_definefun_sat_2.smt2");
}

#[test]
fn test_loops_1() {
    assert_satisfiable("excluded/from_regexbenchmarks/deadloop1_sat.smt2");
}

#[test]
fn test_not1() {
    assert_satisfiable("benchmarks/simple_not_sat_1.smt2");
}

#[test]
fn test_not2() {
    assert_satisfiable("benchmarks/simple_not_sat_2.smt2");
}

/* Tests - range tests

   Currently unknown what tests support these

*/
#[ignore]
#[test]
fn test_range_intersect1() {
    assert_satisfiable("benchmarks/range/intersectranges_sat.smt2");
}

#[test]
fn test_range_intersect2() {
    assert_unsatisfiable("benchmarks/range/intersectranges_unsat.smt2");
}

#[ignore]
#[test]
fn test_range_outside() {
    assert_unsatisfiable("benchmarks/range/range_out_of_range_unsat.smt2");
}

#[ignore]
#[test]
fn test_range_outside_2() {
    assert_unsatisfiable("benchmarks/range/rangecomplement_unsat.smt2");
}

#[test]
fn test_range_simple() {
    assert_satisfiable("benchmarks/range/range_sat.smt2");
}

#[test]
fn test_union_range1() {
    assert_satisfiable("benchmarks/range/union_ranges_sat.smt2");
}

#[ignore]
#[test]
fn test_union_range2() {
    assert_satisfiable("benchmarks/range/union_ranges_2_sat.smt2");
}

#[ignore]
#[test]
fn test_union_range3() {
    assert_unsatisfiable("benchmarks/range/union_ranges_unsat.smt2");
}
/*
    Tests - medium cases

    Only Antimirov can currently handle these
    (testing default solver only)
*/

#[test]
fn test_date_2() {
    fn create_case_insensitive(word: &str) -> Rc<GenRegex> {
        //init first character of word
        let first_char = word.chars().next().unwrap();
        let mut curr_regex = GenRegex::str_to_re(&first_char.to_uppercase().to_string());
        let lower = GenRegex::str_to_re(&first_char.to_lowercase().to_string());
        curr_regex = GenRegex::union(&curr_regex, &lower);

        //iterate over word and add union of upper and lowercase versions
        for c in word[1..].chars() {
            let lower = GenRegex::str_to_re(&c.to_lowercase().to_string());
            let upper = GenRegex::str_to_re(&c.to_uppercase().to_string());
            let char_union = GenRegex::union(&upper, &lower);

            curr_regex = GenRegex::concat(&curr_regex, &char_union);
        }

        curr_regex
    }
    let smt_result = parse_smtlib_file("excluded/from_regexbenchmarks/date2_sat.smt2");
    println!("Parsed s-expression: {:?}\n", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let dot_star = GenRegex::sigma_star();
    let mut days_of_the_week: Vec<Rc<GenRegex>> = Vec::new();
    days_of_the_week.push(create_case_insensitive("monday"));
    days_of_the_week.push(create_case_insensitive("tuesday"));
    days_of_the_week.push(create_case_insensitive("wednesday"));
    days_of_the_week.push(create_case_insensitive("thursday"));
    days_of_the_week.push(create_case_insensitive("friday"));
    days_of_the_week.push(create_case_insensitive("saturday"));
    days_of_the_week.push(create_case_insensitive("sunday"));
    let mut union_days = days_of_the_week[0].clone();
    for v in &days_of_the_week[1..] {
        union_days = GenRegex::union(&union_days, v);
    }
    let mut months: Vec<Rc<GenRegex>> = Vec::new();
    months.push(create_case_insensitive("january"));
    months.push(create_case_insensitive("february"));
    months.push(create_case_insensitive("march"));
    months.push(create_case_insensitive("april"));
    months.push(create_case_insensitive("may"));
    months.push(create_case_insensitive("june"));
    months.push(create_case_insensitive("july"));
    months.push(create_case_insensitive("august"));
    months.push(create_case_insensitive("september"));
    months.push(create_case_insensitive("october"));
    months.push(create_case_insensitive("november"));
    months.push(create_case_insensitive("december"));
    let mut union_months = months[0].clone();
    for v in &months[1..] {
        union_months = GenRegex::union(&union_months, v);
    }
    let first = GenRegex::concat(
        &GenRegex::concat(&dot_star.clone(), &union_days),
        &dot_star.clone(),
    );
    let second = GenRegex::concat(
        &GenRegex::concat(&dot_star.clone(), &union_months),
        &dot_star.clone(),
    );
    //let third = GenRegex::star(&GenRegex::re_range(&'!', &'~'));
    let regex = GenRegex::intersect(&first, &second);
    let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

    assert_eq!(gen_regex_unwrapped, expected);

    assert_satisfiable_regex_default_only(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_passw_unsat1() {
    let smt_result = parse_smtlib_file("excluded/from_regexbenchmarks/passw_unsat1.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let dot_star = GenRegex::sigma_star();
    let first = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('a', 'z')),
        &dot_star,
    );
    let second = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('A', 'Z')),
        &dot_star,
    );
    let third = GenRegex::concat(
        &GenRegex::concat(&dot_star, &GenRegex::re_range('0', '9')),
        &dot_star,
    );
    let fourth = GenRegex::star(&GenRegex::re_range(':', '~'));
    let regex = GenRegex::intersect(
        &GenRegex::intersect(&GenRegex::intersect(&first, &second), &third),
        &fourth,
    );
    let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

    assert_eq!(gen_regex_unwrapped, expected);

    assert_unsatisfiable_regex_default_only(&Rc::new(gen_regex_unwrapped.clone()));
}

#[test]
fn test_let_5() {
    assert_satisfiable_default_only("excluded/from_regexbenchmarks/date_format_days_sat.smt2");
}

#[test]
fn test_loops_2() {
    assert_unsatisfiable_default_only("excluded/from_regexbenchmarks/det_blowup_unsat_3.smt2");
}

#[test]
fn test_loops_3() {
    assert_unsatisfiable_default_only("excluded/from_regexbenchmarks/inter_mod2_unsat.smt2");
}

/*
    Tests - hardest cases, currently ignored

    No solvers can currently handle these
*/

// TODO: Equality not supported for now
#[ignore]
#[test]
fn test_equality() {
    let smt_result = parse_smtlib_file("excluded/from_regexbenchmarks/passw_eq_sat1.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();

    let dot_star = GenRegex::sigma_star();
    let one = GenRegex::concat_many(&vec![
        dot_star.clone(),
        GenRegex::re_range('a', 'z'),
        dot_star.clone(),
    ]);
    let two = GenRegex::concat_many(&vec![
        dot_star.clone(),
        GenRegex::re_range('0', '9'),
        dot_star.clone(),
    ]);
    let three = GenRegex::concat_many(&vec![
        dot_star.clone(),
        GenRegex::re_range('A', 'Z'),
        dot_star.clone(),
    ]);
    let four = GenRegex::re_loop(8, 20, &GenRegex::create_sigma());
    let five = GenRegex::star(&GenRegex::re_range('A', 'z'));
    let together = GenRegex::intersect_many(&vec![
        one.clone(),
        two.clone(),
        three.clone(),
        four.clone(),
        five.clone(),
    ]);
    let eq1 = GenRegex::intersect(&GenRegex::empty_set(), &GenRegex::complement(&together));
    let eq2 = GenRegex::intersect(&GenRegex::complement(&GenRegex::empty_set()), &together);
    let expected = GenRegex::Union(eq1, eq2);
    assert_eq!(gen_regex_unwrapped, expected);
    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped));
}

// TODO: Equality not supported for now
#[ignore]
#[test]
fn test_disequality() {
    assert_unsatisfiable("benchmarks/simple_neq_unsat.smt2")
}

// TODO
#[ignore]
#[test]
fn test_hex_code() {
    let smt_result = parse_smtlib_file("benchmarks/hexcode_sat.smt2");
    println!("Parsed s-expression: {:?}", smt_result);

    assert!(smt_result.is_ok());
    let s_expr = smt_result.unwrap();

    // Parse the s-expression as a GenRegex
    let mut parser = SmtParser::new();
    let gen_regex = parser.parse_s_expr(&s_expr);
    println!("Parsed GenRegex: {:?}", gen_regex);

    assert!(gen_regex.is_ok());
    let gen_regex_unwrapped = gen_regex.unwrap();
    assert_satisfiable_regex(&Rc::new(gen_regex_unwrapped.clone()));
}

// TODO: not working for Brzozowski
#[ignore]
#[test]
fn unicode_hex_test() {
    assert_satisfiable("benchmarks/hex_syntax_test_sat.smt2");
}

// Quite slow
#[ignore]
#[test]
fn intersect_test1() {
    assert_satisfiable("excluded/from_regexbenchmarks/intersect_0_0_sat.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_usr_2() {
    assert_satisfiable("benchmarks/usr_2_sat.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_passw_complement_1() {
    assert_satisfiable_default_only("excluded/from_regexbenchmarks/passw_complex_sat_1.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_passw_complement_2() {
    assert_satisfiable_default_only("excluded/from_regexbenchmarks/passw_complex_sat_2.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_passw_complement_simpler() {
    assert_satisfiable_default_only("excluded/from_regexbenchmarks/passw_simpler_sat_1.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_passw_complement_simplest() {
    assert_satisfiable_default_only("excluded/from_regexbenchmarks/passw_simplest_sat_1.smt2");
}

// TODO: Diverging
#[ignore]
#[test]
fn test_passw_complement_4() {
    assert_unsatisfiable_default_only("excluded/from_regexbenchmarks/passw_very_complex_unsat.smt2");
}

// TODO: Returning wrong answer for Antimirov
#[ignore]
#[test]
fn test_zelkova_ex() {
    assert_unsatisfiable_default_only("benchmarks/zelkova_unsat.smt2")
}
