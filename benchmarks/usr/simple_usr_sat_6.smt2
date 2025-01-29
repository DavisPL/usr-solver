(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (str.to_re "batbatbat")  (re.comp(re.++ (str.to_re w) (str.to_re w) (str.to_re w))))))
(check-sat)
