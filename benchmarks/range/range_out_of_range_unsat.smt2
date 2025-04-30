(declare-fun x () String)

(assert (str.in.re x (re.inter (re.range "a" "c") (str.to.re "d"))))

(check-sat)
