pub(super) fn render_mainline_summary(
    status: &str,
    latest_result: &str,
    next_step: &str,
    primary_action: &str,
    extra_after_status: &[(&str, &str)],
) -> String {
    let mut lines = vec![format!("当前判断：{status}")];
    for (label, value) in extra_after_status {
        lines.push(format!("{label}：{value}"));
    }
    lines.push(format!("最近结果：{latest_result}"));
    lines.push(format!("下一步：{next_step}"));
    lines.push(primary_action.to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainline_summary_keeps_shared_order_without_extra_lines() {
        let text = render_mainline_summary(
            "当前可直接 Apply",
            "暂无",
            "进入 Apply 预览",
            "主动作：Preview Apply",
            &[],
        );

        let status_pos = text.find("当前判断：当前可直接 Apply").unwrap();
        let result_pos = text.find("最近结果：暂无").unwrap();
        let next_step_pos = text.find("下一步：进入 Apply 预览").unwrap();
        let primary_pos = text.find("主动作：Preview Apply").unwrap();

        assert!(status_pos < result_pos);
        assert!(result_pos < next_step_pos);
        assert!(next_step_pos < primary_pos);
    }

    #[test]
    fn mainline_summary_keeps_reason_between_status_and_latest_result() {
        let text = render_mainline_summary(
            "当前组合应先复查",
            "flake check 已完成。",
            "先看健康详情",
            "主动作：先看健康详情，再决定是否执行当前检查",
            &[("原因", "默认主路径先确认健康和执行门槛")],
        );

        let status_pos = text.find("当前判断：当前组合应先复查").unwrap();
        let reason_pos = text.find("原因：默认主路径先确认健康和执行门槛").unwrap();
        let result_pos = text.find("最近结果：flake check 已完成。").unwrap();
        let next_step_pos = text.find("下一步：先看健康详情").unwrap();
        let primary_pos = text
            .find("主动作：先看健康详情，再决定是否执行当前检查")
            .unwrap();

        assert!(status_pos < reason_pos);
        assert!(reason_pos < result_pos);
        assert!(result_pos < next_step_pos);
        assert!(next_step_pos < primary_pos);
    }
}
