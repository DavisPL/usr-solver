;Translated
(set-logic QF_S)

(declare-const s1 String)
(declare-const s2 String)
(declare-const x String)

(assert (str.in_re s2 (re.++ (str.to_re "el") (str.to_re "lo") )))

(assert (str.in_re x
(re.inter

(re.++
	(str.to_re s1)
	(str.to_re s2)

)
(re.comp (str.to_re "hello"))

)
))

(check-sat)

