;StringVar w
;Membership in wwwwwww

(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.++ (str.to_re w) (str.to_re w) (str.to_re w) (str.to_re w) (str.to_re w) (str.to_re w))))
(check-sat)