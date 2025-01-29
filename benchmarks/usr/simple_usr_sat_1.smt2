;StringVar w
;Membership in waw
;(str.to_re w)

(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.++ (str.to_re w) (str.to_re "a") (str.to_re w))))
(check-sat)