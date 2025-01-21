(declare-const x String)
(assert (str.in_re x (re.range "\u{AA}" "\u{AA}")))
(check-sat)