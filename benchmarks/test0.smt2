(declare-const x String)
(assert (str.in_re x
    (re.comp (re.++ (re.++ re.all (re.union (str.to_re "0") (str.to_re "0"))) re.all))
))
(check-sat)
