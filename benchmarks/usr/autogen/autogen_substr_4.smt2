(declare-const s1 String)
(declare-const y String)

;(assert (= "ht" (str.substr s1 17 2)))
;(assert (= "z2ss" (str.substr s1 13 15)))
;(assert (= "9" (str.substr s1 6 1)))

(assert
    (str.in_re y
        (re.++
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "ht") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "ht")
                        (re.++ re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "ht"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "ht")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "z2ss") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "z2ss")
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "z2ss"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "z2ss")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "9") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "9")
                        re.allchar
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "9"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "9")
                        (str.to_re "")
                    )
                )
            )
        )
    )
)
(check-sat)
