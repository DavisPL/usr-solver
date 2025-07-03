(set-logic QF_S)
(declare-const x String)
(assert (str.in_re x (re.++
    (str.to_re "ab")
    re.all
)))
(assert (not (str.in_re x (re.++
    (str.to_re "a")
    re.all
))))

(check-sat)
