(declare-const x String)
(assert (str.in_re x (re.range (_ char #x0) "/")))
(check-sat)