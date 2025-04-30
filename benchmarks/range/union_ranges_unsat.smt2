(set-logic QF_S)

(declare-fun x () String)

(assert (and
  (str.in_re x (re.union (re.range "a" "c") (re.range "x" "z")))

  (not (str.in_re x (str.to_re "a")))
  (not (str.in_re x (str.to_re "b")))
  (not (str.in_re x (str.to_re "c")))
  (not (str.in_re x (str.to_re "x")))
  (not (str.in_re x (str.to_re "y")))
  (not (str.in_re x (str.to_re "z")))
))

(check-sat)

