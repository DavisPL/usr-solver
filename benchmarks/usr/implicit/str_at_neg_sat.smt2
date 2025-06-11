(set-logic QF_S)

(declare-const s1 String)

(assert (not (= (str.at s1 0) "a")))

(assert (str.in_re s1 (re.++ (re.* re.allchar) (str.to_re "ab") (re.* re.allchar) )))

(assert (= (str.at s1 1) "b"))

(check-sat)


