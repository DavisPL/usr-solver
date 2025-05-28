; Translated
(set-logic QF_S)

(declare-const s1 String)
(declare-const y String)

(assert (str.in_re y (re.++

	(re.inter
		(str.to_re y)
			(str.to_re s1)
			(re.++
				(str.to_re "a")
				(re.* re.allchar)
			)


	)
)))
(assert (str.in_re y (re.++

	(re.inter
		(str.to_re y)
			(str.to_re s1)
			(re.++
				re.allchar
				(str.to_re "b")
				(re.* re.allchar)
			)

	)

)

))


(check-sat)

