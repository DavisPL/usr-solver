(set-logic QF_S)

(declare-fun x () String)

(assert (and
  (str.in.re x (re.range "m" "p"))
  (str.in.re x (re.union (str.to.re "l") (str.to.re "p")))
))

(check-sat)

