(declare-const x String)
(assert (str.in_re x
    (re.inter
        (re.comp (re.++ (re.++ re.all (re.union (str.to_re "01") (str.to_re "12"))) re.all))
        ((_ re.loop 7 10) re.allchar)
    )
))
(check-sat)
