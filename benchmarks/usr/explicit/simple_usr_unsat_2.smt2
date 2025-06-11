;StringVar w1 w2
;Membership in (w1)a \cap (w1)(w2)aa(w2)

(declare-fun w1 () String)
(declare-fun w2 () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (str.to_re w1) (str.to_re "a")) (re.++ (str.to_re w1) (str.to_re w2) (str.to_re "aa") (str.to_re w2)))))
(check-sat)