; re.all TEST:
(declare-const x String)
(assert (str.in_re x (re.++ re.all (re.union (str.to_re "a") (str.to_re "b")))))
(check-sat)

;EXPECTED
;x \cap .*(a U b)