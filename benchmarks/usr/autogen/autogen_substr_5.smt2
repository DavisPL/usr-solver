(declare-const s1 String)
(declare-const y String)

;(assert (= "p8" (str.substr s1 16 2)))
;(assert (= "4k1g" (str.substr s1 14 4)))

(assert
    (str.in_re y
        (re.++
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "p8") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "p8")
                        (re.++ re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "p8"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "p8")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "4k1g") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "4k1g")
                        (re.++ re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "4k1g"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "4k1g")
                        (str.to_re "")
                    )
                )
            )
        )
    )
)
(check-sat)
