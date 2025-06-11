(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (re.comp(str.to_re "c"))(str.to_re "atbatbat"))  (re.++ (str.to_re w) (str.to_re w) (str.to_re w)))))
(check-sat)
