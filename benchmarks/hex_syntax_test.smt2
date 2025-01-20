(declare-const x String)
(assert (str.in_re x (re.range "\u{aa}" "\u{aa}")))
(check-sat)