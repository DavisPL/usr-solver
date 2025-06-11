;StringVar w1, w2
;Membership in (w1)(w2) \cap (w2)(w1)

(declare-fun w1 () String)
(declare-fun w2 () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (str.to_re w1) (str.to_re w2)) (re.++ (str.to_re w2) (str.to_re w1)))))
(check-sat)