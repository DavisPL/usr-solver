; Translated
(set-logic QF_S)
(declare-const s1 String)
(declare-const y String)

(assert (str.in_re y 

	(re.inter
		(str.to_re s1)
		(re.comp(re.++
			(str.to_re "a")
			(re.* re.allchar)
		))
)))
(assert (str.in_re y

	(re.inter
		(str.to_re s1)
		(re.++
			(re.* re.allchar)
			(str.to_re "ab")
			(re.* re.allchar)
		)
	)

))
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

