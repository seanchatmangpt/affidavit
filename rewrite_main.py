import re

with open("thesis/main.tex", "r") as f:
    content = f.read()

# Remove everything from PART I to the end
parts_start = content.find("% PART I:")
if parts_start != -1:
    # Go back a few lines to catch the header
    parts_start = content.rfind("% ----", 0, parts_start)

header = content[:parts_start]

new_parts = """
% ------------------------------------------------------------------------------
% PART I: THEORETICAL FOUNDATIONS
% ------------------------------------------------------------------------------
\part{Theoretical Foundations}

\input{chapters/01_intro_part1.tex}
\input{chapters/01_intro_part2.tex}
\input{chapters/02_lit_review_pm.tex}
\input{chapters/02_lit_review_crypto.tex}
\input{chapters/02_lit_review_formal.tex}
\input{chapters/03_theory_chatman.tex}
\input{chapters/03_theory_ontology.tex}
\input{chapters/03_theory_tla.tex}

% ------------------------------------------------------------------------------
% PART II: THE MANUFACTURING PIPELINE
% ------------------------------------------------------------------------------
\part{The Cryptographic Manufacturing Pipeline}

\input{chapters/04_impl_core.tex}
\input{chapters/04_impl_scale.tex}
\input{chapters/04_impl_qol.tex}

% ------------------------------------------------------------------------------
% PART III: EMPIRICAL EVALUATION AND CONCLUSION
% ------------------------------------------------------------------------------
\part{Empirical Evaluation and Conclusion}

\input{chapters/05_eval_perf.tex}
\input{chapters/05_eval_chaos.tex}
\input{chapters/05_eval_fuzz.tex}
\input{chapters/06_conclusion.tex}

% ------------------------------------------------------------------------------
% BACKMATTER: Appendices, Bibliography, Indices
% ------------------------------------------------------------------------------
\\backmatter

\\begin{appendices}
\\input{appendices/A_defense.tex}
\\input{appendices/B_typestate.tex}
\\end{appendices}

\\cleardoublepage
\\phantomsection
\\addcontentsline{toc}{chapter}{\\bibname}
\\printbibliography

\\end{document}
"""

with open("thesis/main.tex", "w") as f:
    f.write(header + new_parts)
