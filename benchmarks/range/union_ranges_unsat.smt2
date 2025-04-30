(set-logic QF_S)

(declare-fun x () String)

(assert (and
  (str.in_re x (re.union (re.range "a" "c") (re.range "x" "z")))

  (not (str.in_re x (str.to.re "a")))
  (not (str.in_re x (str.to.re "b")))
  (not (str.in_re x (str.to.re "c")))
  (not (str.in_re x (str.to.re "x")))
  (not (str.in_re x (str.to.re "y")))
  (not (str.in_re x (str.to.re "z")))
))

(check-sat)

