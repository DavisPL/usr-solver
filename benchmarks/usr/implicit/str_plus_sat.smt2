(set-logic QF_S)

(declare-const s1 String)
(declare-const s2 String)

(assert (str.in_re s2 (re.++ (str.to_re "el") (re.* re.allchar) )))

(assert (= (str.++ s1 s2) "hello"))

(check-sat)

