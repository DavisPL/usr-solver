(declare-fun x () String)

(assert (str.in_re x (re.inter (re.range "a" "c") (str.to_re "d"))))

(check-sat)
