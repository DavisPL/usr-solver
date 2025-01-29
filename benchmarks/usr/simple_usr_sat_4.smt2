(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (str.to_re w) (str.to_re w) (str.to_re "bat")) (re.++ (str.to_re w) (str.to_re w) (str.to_re w)))))
(check-sat)