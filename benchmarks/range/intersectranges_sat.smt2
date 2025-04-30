(set-logic QF_S)

(declare-fun x () String)

(assert (str.in_re x (re.inter (re.range "d" "h") (re.range "f" "k"))))

(check-sat)
