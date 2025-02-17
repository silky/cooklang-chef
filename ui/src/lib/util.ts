import type { Report, Step } from './types';

export function isValid<T>(report: Report<T>) {
	return report.value !== null && report.errors.length === 0;
}

export function displayName<T extends { name: string; alias: string | null }>(component: T) {
	return component.alias ?? component.name;
}

export function isTextStep(step: Step) {
	return step.number === null;
}

export function formatTime(minutes: number) {
	let hours = Math.trunc(minutes / 60);
	minutes %= 60;
	const days = Math.trunc(hours / 24);
	hours %= 24;

	const parts = [];
	// TODO maybe not construct formatters in every call
	if (days > 0) {
		parts.push(new Intl.NumberFormat(undefined, { style: 'unit', unit: 'day' }).format(days));
	}
	if (hours > 0) {
		parts.push(new Intl.NumberFormat(undefined, { style: 'unit', unit: 'hour' }).format(hours));
	}
	if (minutes > 0) {
		parts.push(new Intl.NumberFormat(undefined, { style: 'unit', unit: 'minute' }).format(minutes));
	}
	return parts.join(' ');
}
