;StringVar w
;Membership in cwt \cap caw

(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (str.to_re w) (str.to_re "a")) (re.++ (str.to_re w) (str.to_re "aa")))))
(check-sat)