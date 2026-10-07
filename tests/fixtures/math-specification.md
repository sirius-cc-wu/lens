# Mathematical Specification & Timing Analysis

Physical layer wire limits, frame durations, and timing budgets for the controller interface:

## Timing Limits & Wire Budgets

The nominal frame transmission time is defined as:

$$
\begin{aligned}
T_{\text{frame}} &= N_{\text{bits}} \times t_{\text{bit}} \\
&= 128 \times 1.25\,\mu\text{s} = 160\,\mu\text{s}
\end{aligned}
$$

where $t_{\text{bit}} = 1.25\,\mu\text{s}$ and total jitter margin is bounded by $\Delta t \le 15\,\mu\text{s}$.

## State Transitions & Queue Rates

State progression across the IPCF interface follows the relation:

$$\text{State}_A \xrightarrow{1} \text{State}_B$$

The packet processing headroom is computed as:

```math
f_{\text{loop}} = \frac{4\,\text{pkts}}{\text{ms}} \implies 4\,\text{kHz}
```

## System Parameters Table

| Parameter | Notation | Target Limit |
| :--- | :--- | :--- |
| Frame Duration | $T_{\text{frame}}$ | $125\,\mu\text{s}$ |
| Energy Consumption | $E = mc^2$ | $15\,\text{mJ}$ |
| Propagation Delay | $t_{\text{prop}}$ | $45\,\text{ns}$ |
| Magnitude | $\lvert x \rvert$ | $\le 1$ |

## Financial & Environment Constraints

The procurement budget per unit is $10 and $20 for expansion slots.
A price range is written as \$5-\$10.
Escaped references like \$50 and \$100 are also preserved as literal currency values.

Shell environment variables should be referenced via inline code spans: `echo $PATH`, `$(CC)$(FLAGS)`, and `$VAR` without math conversion.
