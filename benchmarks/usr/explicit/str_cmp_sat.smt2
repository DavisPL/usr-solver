(set-logic QF_S) 

(declare-const s1 String)

(assert (str.in_re s1 (re.++ (str.to_re "ab") (re.* re.allchar))))

(assert (str.< s1 "abaaaa"))

(check-sat)

(get-model)

