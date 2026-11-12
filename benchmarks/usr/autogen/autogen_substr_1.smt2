(declare-const s1 String)
(declare-const y String)

;(assert (= "y" (str.substr s1 7 1)))

(assert
    (str.in_re y
        (re.union
            (re.++
                (re.inter
                    (str.to_re s1)
                    (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "y") (re.* re.allchar))
                )
                (re.inter
                    (str.to_re "y")
                    re.allchar
                )
                (re.inter
                    (str.to_re s1)
                    (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                )
            )
            (re.++
                (re.inter
                    (str.to_re s1)
                    (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "y"))
                )
                (re.inter
                    (str.to_re s1)
                    (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar (re.* re.allchar))))
                )
            )
            (re.++
                (re.inter
                    (str.to_re s1)
                    (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                )
                (re.inter
                    (str.to_re "y")
                    (str.to_re "")
                )
            )
        )
    )
)
(check-sat)
