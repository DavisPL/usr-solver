(declare-const x String)
(assert (not (str.in_re x (re.++ (str.to_re "a") (str.to_re "b")))))
(check-sat)
