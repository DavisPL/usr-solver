;StringVar w
;Membership in aw \cap wb

(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (str.to_re "a") (str.to_re w)) (re.++ (str.to_re w) (str.to_re "b")))))
(check-sat)
