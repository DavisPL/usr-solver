(set-logic QF_S)

(declare-fun x () String)

(assert (and
  (str.in_re x (re.range "m" "p"))
  (str.in_re x (re.union (str.to_re "l") (str.to_re "p")))
))

(check-sat)

