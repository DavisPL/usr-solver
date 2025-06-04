;Translated
(set-logic QF_S)

(declare-const s1 String)
(declare-const y String)
(declare-const z String)

;(assert (= "pine" (str.substr s1 2 4)))

(assert (str.in_re y
	(re.union
		(re.++
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					(str.to_re "pine")
					(re.* re.allchar)
				)

			)
			(re.inter
				(str.to_re "pine")
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
				)
			)
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					(re.* re.allchar)
				)

			)
		)
		(re.++
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					(str.to_re "pine")
				)

			)
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					(re.comp
						(re.++
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							(re.* re.allchar)
						)
					)
				)
			)
		)
		(re.++
			(re.inter
				(str.to_re s1)
				(re.comp
					(re.++
						re.allchar
						re.allchar
						(re.* re.allchar)
					)
				)

			)
			(re.inter
				(str.to_re "pine")
				(str.to_re "")
			)
		)
	)
)
)

(assert (str.in_re z
	(re.union
		(re.++
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					(str.to_re "rine")
					(re.* re.allchar)
				)

			)
			(re.inter
				(str.to_re "rine")
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
				)
			)
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					(re.* re.allchar)
				)

			)
		)
		(re.++
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					(str.to_re "rine")
				)

			)
			(re.inter
				(str.to_re s1)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					re.allchar
					(re.comp
						(re.++
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							re.allchar
							(re.* re.allchar)
						)
					)
				)
			)
		)
		(re.++
			(re.inter
				(str.to_re s1)
				(re.comp
					(re.++
						re.allchar
						re.allchar
						re.allchar
						re.allchar
						re.allchar
						re.allchar
						re.allchar
						re.allchar
						(re.* re.allchar)
					)
				)

			)
			(re.inter
				(str.to_re "rine")
				(str.to_re "")
			)
		)
	)
)
)


; (assert (= "rine" (str.substr s1 8 13)))

(check-sat)

;(get-model)

