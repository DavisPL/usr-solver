(declare-fun w () String)
(declare-const x String)
(assert (str.in_re x (re.inter (re.++ (re.comp (str.to_re "b")) re.all (str.to_re "t")) (re.++ (str.to_re w) (str.to_re w) (str.to_re w)))))
(check-sat)
