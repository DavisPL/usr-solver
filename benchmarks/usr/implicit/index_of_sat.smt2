;Translated
(set-logic QF_S)

(declare-fun s () String)
(declare-fun s1 () String)
(declare-fun s2 () String)
(declare-fun s3 () String)
(declare-fun x () String)

(assert (str.in_re s3 re.allchar))
(assert (str.in_re x (
	re.++
	 (re.inter 
				(str.to_re s)
				(re.++
					re.allchar
					re.allchar
					re.allchar
					(str.to_re s1)
					(str.to_re s2)
					(str.to_re s3)
					(re.* re.allchar)

				)

			  

		)

	 (re.inter 
			(re.++ 
				(str.to_re s1)
				(str.to_re s2)
			)
			(re.comp
				(re.++
					(re.* re.allchar)
					(str.to_re "abc")
					(re.* re.allchar)


				)

			)
		

			   



	)
(re.inter 
		(str.to_re s1)
		(re.++
			re.allchar
			re.allchar
		)

		  )
(re.inter
	(re.++
		(str.to_re s2)
		(str.to_re s3)

	)
	(str.to_re "abc")


		 )



	)


)




)

(check-sat)

