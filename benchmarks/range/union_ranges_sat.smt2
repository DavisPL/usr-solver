(set-logic QF_S)

(declare-fun x () String)

(assert (str.in_re x (re.union (re.range "a" "c") (re.range "x" "z"))))

(check-sat)
